//! Crate name parsing.
use crate::errors::*;
use crate::Dependency;
use crate::{get_manifest_from_path, get_manifest_from_url};

/// A crate specifier. This can be a plain name (e.g. `docopt`), a name and a versionreq (e.g.
/// `docopt@^0.8`), a URL, or a path.
#[derive(Debug)]
pub struct CrateName<'a>(&'a str);

impl<'a> CrateName<'a> {
    /// Create a new `CrateName`
    pub fn new(name: &'a str) -> Self {
        CrateName(name)
    }

    /// Get crate name
    pub fn name(&self) -> &str {
        self.0
    }

    /// Does this specify a versionreq?
    pub fn has_version(&self) -> bool {
        self.0.contains('@')
    }

    /// If this crate specifier includes a version (e.g. `docopt@0.8`), extract the name and
    /// version.
    pub fn parse_version_or_features(&self) -> Result<Option<Dependency>> {
        let mut name = self.0;
        let mut features: Option<&str> = None;
        let mut version: Option<&str> = None;
        if self.0.contains('+') {
            let xs: Vec<_> = self.0.splitn(2, '+').collect();
            name = xs[0];
            features = Some(xs[1]);
        }
        if self.has_version() {
            let xs: Vec<_> = self.0.splitn(2, '@').collect();
            name = xs[0];
            version = Some(xs[1]);
            semver::VersionReq::parse(version.unwrap()).chain_err(|| "Invalid crate version requirement")?;
        }
        if features.is_some() || version.is_some() {
            let mut dep = Dependency::new(name);
            if let Some(f) = features {
                dep = dep.set_features(Some(f.split(',').map(|s| s.to_string()).collect()));
            }
            if let Some(v) = version {
                dep = dep.set_version(v);
            }
            Ok(Some(dep))
        } else {
            Ok(None)
        }
    }

    /// Will parse this crate name on the assumption that it is a URI.
    pub fn parse_crate_name_from_uri(&self) -> Result<Option<Dependency>> {
        if let Some(manifest) = get_manifest_from_url(self.0)? {
            let crate_name = manifest.package_name()?;
            let available_features = manifest.features()?;
            Ok(Some(
                Dependency::new(crate_name)
                    .set_git(self.0, None)
                    .set_available_features(available_features),
            ))
        } else if self.is_path() {
            let path = std::path::Path::new(self.0);
            let manifest = get_manifest_from_path(path)?;
            let crate_name = manifest.package_name()?;
            let path = dunce::canonicalize(path)?;
            let available_features = manifest.features()?;
            Ok(Some(
                Dependency::new(crate_name)
                    .set_path(path)
                    .set_available_features(available_features),
            ))
        } else {
            Ok(None)
        }
    }

    fn is_path(&self) -> bool {
        // FIXME: how else can we check if the name is a (possibly invalid) path?
        self.0.contains('.') || self.0.contains('/') || self.0.contains('\\')
    }
}
