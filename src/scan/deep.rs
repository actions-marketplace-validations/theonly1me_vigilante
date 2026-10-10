use super::{Scanner, Selection};
use crate::Error;
use crate::config::LanguageEngine;
use crate::deep::Analysis;
use crate::go::GoRequest;
use crate::java::{self, JavaRequest};
use crate::javascript::{self, JavaScriptRequest};
use crate::python::{self, PythonRequest};

impl Scanner<'_> {
    pub(super) fn deep(&self, selection: &Selection) -> Result<Analysis, Error> {
        let mut analysis = Analysis::default();
        if let Some(go) = &self.go {
            let changed = match selection {
                Selection::Everything => None,
                Selection::Changes(changes) => Some(&changes.paths),
            };
            analysis = go.analyze(GoRequest {
                changed,
                location: self.location,
                config: self.config,
            })?;
        }
        if self.config.javascript_engine == LanguageEngine::Deep {
            let found = javascript::analyze(JavaScriptRequest {
                tree: self.tree,
                config: self.config,
                rules: &self.rules,
                scope_of: &|path| self.scopes.code_scope_of(path),
                selection,
            })?;
            analysis.sightings.extend(found.sightings);
            analysis.reaches.extend(found.reaches);
            analysis.skipped.extend(found.skipped);
        }
        if self.config.python_engine == LanguageEngine::Deep {
            let found = python::analyze(PythonRequest {
                tree: self.tree,
                config: self.config,
                rules: &self.rules,
                scope_of: &|path| self.scopes.code_scope_of(path),
                selection,
            })?;
            analysis.sightings.extend(found.sightings);
            analysis.reaches.extend(found.reaches);
            analysis.skipped.extend(found.skipped);
        }
        if self.config.java_engine == LanguageEngine::Deep {
            let found = java::analyze(JavaRequest {
                tree: self.tree,
                config: self.config,
                rules: &self.rules,
                scope_of: &|path| self.scopes.code_scope_of(path),
                selection,
            })?;
            analysis.sightings.extend(found.sightings);
            analysis.reaches.extend(found.reaches);
            analysis.skipped.extend(found.skipped);
        }
        Ok(analysis)
    }
}
