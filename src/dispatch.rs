use std::collections::HashMap;

use crate::lang::oml::OmlHandler;
use crate::lang::wfg::WfgHandler;
use crate::lang::wfl::WflHandler;
use crate::lang::wfs::WfsHandler;
use crate::lang::wpl::WplHandler;
use crate::lang::LangHandler;

/// Dispatches LSP requests to the appropriate language handler based on
/// language ID or file extension.
pub struct Dispatcher {
    by_lang_id: HashMap<String, Box<dyn LangHandler>>,
    by_extension: HashMap<String, String>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn LangHandler>> = vec![
            Box::new(WflHandler),
            Box::new(WfsHandler),
            Box::new(WplHandler),
            Box::new(OmlHandler),
            Box::new(WfgHandler),
        ];

        let mut by_lang_id = HashMap::new();
        let mut by_extension = HashMap::new();

        for handler in handlers {
            let lang_id = handler.lang_id().to_string();
            for ext in handler.extensions() {
                by_extension.insert(ext.to_string(), lang_id.clone());
            }
            by_lang_id.insert(lang_id, handler);
        }

        Self {
            by_lang_id,
            by_extension,
        }
    }

    pub fn handler_by_lang_id(&self, lang_id: &str) -> Option<&dyn LangHandler> {
        self.by_lang_id.get(lang_id).map(|h| h.as_ref())
    }

    /// Infer the language ID from a document URI's file extension.
    pub fn lang_id_for_uri(&self, uri: &lsp_types::Url) -> Option<&str> {
        let path = uri.path();
        let ext = path.rsplit('.').next()?;
        self.by_extension.get(ext).map(|s| s.as_str())
    }
}
