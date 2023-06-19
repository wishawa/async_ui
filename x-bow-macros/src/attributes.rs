use proc_macro2::Ident;
use syn::Attribute;

pub(crate) const ATTRIBUTE_PATH: &str = "x_bow";
pub(crate) const ATTRIBUTE_TRACK_ALL: &str = "track_all";
pub(crate) const ATTRIBUTE_TRACK: &str = "track";
pub(crate) const ATTRIBUTE_TRACK_DEEP: &str = "deep";
pub(crate) const ATTRIBUTE_TRACK_SHALLOW: &str = "shallow";
pub(crate) const ATTRIBUTE_TRACK_SKIP: &str = "skip";
pub(crate) const ATTRIBUTE_MODULE_PREFIX: &str = "module_prefix";
pub(crate) const ATTRIBUTE_REMOTE_TYPE: &str = "remote_type";

#[derive(Clone, Copy)]
pub(crate) enum TrackMode {
    Deep,
    Shallow,
    Skip,
}

impl TrackMode {
    pub fn from_attributes(attrs: &[Attribute], default: &Self) -> syn::Result<Self> {
        for attr in attrs {
            if attr.path().is_ident(ATTRIBUTE_TRACK) {
                return Ok(match attr.parse_args::<Ident>() {
                    Ok(x) if x == self::ATTRIBUTE_TRACK_DEEP => Self::Deep,
                    Ok(x) if x == self::ATTRIBUTE_TRACK_SKIP => Self::Skip,
                    Ok(x) if x == self::ATTRIBUTE_TRACK_SHALLOW => Self::Shallow,
                    Err(_) => Self::Shallow,
                    Ok(x) => {
                        return Err(syn::Error::new_spanned(
                            x,
                            "unrecognized argument; options are `deep`, `shallow`, and `skip`",
                        ))
                    }
                });
            }
        }
        Ok(*default)
    }
}
