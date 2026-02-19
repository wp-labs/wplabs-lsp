use dashmap::DashMap;
use lsp_types::*;
use tower_lsp::jsonrpc;
use tower_lsp::{Client, LanguageServer};
use tree_sitter::{Parser, Tree};

use crate::capabilities::server_capabilities;
use crate::dispatch::Dispatcher;
use crate::document::DocumentState;
use crate::features;

pub struct WfLsp {
    client: Client,
    documents: DashMap<Url, DocumentState>,
    dispatcher: Dispatcher,
}

impl WfLsp {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            dispatcher: Dispatcher::new(),
        }
    }

    fn parse_with_lang(
        lang: tree_sitter::Language,
        text: &str,
        old_tree: Option<&Tree>,
    ) -> Option<Tree> {
        let mut parser = Parser::new();
        parser.set_language(&lang).ok()?;
        parser.parse(text, old_tree)
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for WfLsp {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo {
                name: "wplabs-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "wplabs-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        // Determine language, prefer language_id from client, fall back to extension
        let lang_id = {
            let lid = &params.text_document.language_id;
            if self.dispatcher.handler_by_lang_id(lid).is_some() {
                lid.clone()
            } else if let Some(inferred) = self.dispatcher.lang_id_for_uri(&uri) {
                inferred.to_string()
            } else {
                return;
            }
        };

        let handler = match self.dispatcher.handler_by_lang_id(&lang_id) {
            Some(h) => h,
            None => return,
        };

        let tree = match Self::parse_with_lang(handler.language(), &text, None) {
            Some(t) => t,
            None => return,
        };

        let diags = features::diagnostics::collect(&tree, &text);

        self.documents.insert(
            uri.clone(),
            DocumentState {
                text,
                tree,
                version,
                lang_id,
            },
        );

        self.client
            .publish_diagnostics(uri, diags, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // For full-sync, use the last content change
        let new_text = match params.content_changes.into_iter().last() {
            Some(change) => change.text,
            None => return,
        };

        let diags = {
            let mut doc = match self.documents.get_mut(&uri) {
                Some(d) => d,
                None => return,
            };

            let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
                Some(h) => h,
                None => return,
            };

            let tree = match Self::parse_with_lang(handler.language(), &new_text, Some(&doc.tree)) {
                Some(t) => t,
                None => return,
            };

            let diags = features::diagnostics::collect(&tree, &new_text);
            doc.text = new_text;
            doc.tree = tree;
            doc.version = version;
            diags
        };

        self.client
            .publish_diagnostics(uri, diags, Some(version))
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        let items = features::completion::complete(handler, &doc.tree, &doc.text, pos);
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        Ok(features::hover::hover(handler, &doc.tree, &doc.text, pos))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        Ok(features::definition::goto_definition(
            handler, &doc.tree, &doc.text, pos, uri,
        ))
    }

    async fn references(&self, params: ReferenceParams) -> jsonrpc::Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let include_decl = params.context.include_declaration;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        let locations = features::references::find_references(
            handler,
            &doc.tree,
            &doc.text,
            pos,
            uri,
            include_decl,
        );

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn rename(&self, params: RenameParams) -> jsonrpc::Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let new_name = &params.new_name;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        Ok(features::rename::rename(
            handler, &doc.tree, &doc.text, pos, uri, new_name,
        ))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        Ok(Some(features::symbols::document_symbols(
            handler, &doc.tree, &doc.text,
        )))
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        let handler = match self.dispatcher.handler_by_lang_id(&doc.lang_id) {
            Some(h) => h,
            None => return Ok(None),
        };

        Ok(features::formatting::format_document(
            handler, &doc.tree, &doc.text,
        ))
    }
}
