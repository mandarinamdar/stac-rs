//! Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.
//!
//! The SpatioTemporal Asset Catalog (STAC) specification provides a common language to describe a range of geospatial information, so it can more easily be indexed and discovered.
//! A 'spatiotemporal asset' is any file that represents information about the earth captured in a certain space and time.
//!
//! This is a Rust implementation of the specification, with associated utilities.
//! Similar projects in other languages include:
//!
//! - Python: [PySTAC](https://pystac.readthedocs.io/en/1.0/)
//! - Go: [go-stac](https://github.com/planetlabs/go-stac)
//! - .NET: [DotNetStac](https://github.com/Terradue/DotNetStac)
//! - PHP: [resto](https://github.com/jjrom/resto)
//!
//! # Data structures
//!
//! STAC has three data structures:
//!
//! - [Item] is a [GeoJSON](http://geojson.org/) [Feature](https://tools.ietf.org/html/rfc7946#section-3.2) augmented with [foreign members](https://tools.ietf.org/html/rfc7946#section-6)
//! - [Catalog] represents a logical group of other `Catalogs`, `Collections`, and `Items`
//! - [Collection] shares all fields with the `Catalog` (with different allowed values for `type` and `stac_extensions`) and adds fields to describe the whole dataset and the included set of `Items`
//!
//! All three are provided as [serde](https://serde.rs/) (de)serializable structures with public attributes.
//! Each structure provides a `new` method that fills most of the object's attributes with sensible defaults:
//!
//! ```
//! use stac::{Item, Catalog, Collection};
//! let item = Item::new("id");
//! let catalog = Catalog::new("id", "description");
//! let collection = Catalog::new("id", "description");
//! ```
//!
//! All attributes of STAC objects are accessible as public members:
//!
//! ```
//! use stac::{Item, Link};
//! let mut item = Item::new("id");
//! assert_eq!(item.id, "id");
//! assert!(item.geometry.is_none());
//! assert!(item.links.is_empty());
//! item.links.push(Link::new("an/href", "a-rel-type"));
//! ```
//!
//! # Reading
//!
//! Synchronous reads from the filesystem are supported via [read]:
//!
//! ```
//! let value: stac::Item = stac::read("data/simple-item.json").unwrap();
//! ```
//!
//! If the [reqwest](https://docs.rs/reqwest/latest/reqwest/) feature is enabled, synchronous reads from urls are also supported:
//!
//! ```
//! #[cfg(feature = "reqwest")]
//! {
//!     let url = "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json";
//!     let item: stac::Item = stac::read(url).unwrap();
//! }
//! ```
//!
//! If `reqwest` is not enabled, reading from a url will return an error:
//!
//! ```
//! #[cfg(not(feature = "reqwest"))]
//! {
//!     let url = "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json";
//!     let error = stac::read::<stac::Item>(url).unwrap_err();
//! }
//! ```
//!
//! ## Hrefs
//!
//! When objects are read from the filesystem or from a remote location, they store the href from which they were read.
//! The href is accessible via the [Href] trait:
//!
//! ```
//! use stac::{Href, Item};
//! let item: Item = stac::read("data/simple-item.json").unwrap();
//! assert!(item.href().as_deref().unwrap().ends_with("data/simple-item.json"));
//! ```
//!
//! # Validation
//!
//! If the `jsonschema` feature is enabled, objects can be validated against their [json-schema](https://json-schema.org/) definitions:
//!
//! ```
//! #[cfg(feature = "jsonschema")]
//! {
//!     use stac::{Item, Validate};
//!     let item = Item::new("an-id");
//!     item.validate().unwrap();
//! }
//! ```
//!
//! See the `validate` module for more examples.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

mod asset;
mod catalog;
mod collection;
mod error;
mod extensions;
mod href;
mod io;
mod item;
mod item_collection;
pub mod link;
pub mod media_type;
#[cfg(feature = "jsonschema")]
pub mod validate;
mod value;

#[cfg(feature = "jsonschema")]
pub use validate::{Validate, Validator};
pub use {
    asset::{Asset, Assets},
    catalog::{Catalog, CATALOG_TYPE},
    collection::{Collection, Extent, Provider, SpatialExtent, TemporalExtent, COLLECTION_TYPE},
    error::Error,
    extensions::Extensions,
    href::{href_to_url, Href},
    io::{read, read_json},
    item::{Item, Properties, ITEM_TYPE},
    item_collection::{ItemCollection, ITEM_COLLECTION_TYPE},
    link::{Link, Links},
    value::Value,
};

/// The default STAC version supported by this library.
pub const STAC_VERSION: &str = "1.0.0";

/// Custom [Result](std::result::Result) type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn deserialize_type<'de, D>(
    deserializer: D,
    expected: &str,
) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::Deserialize;
    let r#type = String::deserialize(deserializer)?;
    if r#type != expected {
        Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(&r#type),
            &expected,
        ))
    } else {
        Ok(r#type)
    }
}

pub(crate) fn serialize_type<S>(
    r#type: &String,
    serializer: S,
    expected: &str,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    use serde::Serialize;
    if r#type != expected {
        Err(serde::ser::Error::custom(format!(
            "type field must be '{}', got: '{}'",
            expected, r#type
        )))
    } else {
        r#type.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    macro_rules! roundtrip {
        ($function:ident, $filename:expr, $object:ident) => {
            #[test]
            fn $function() {
                use assert_json_diff::assert_json_eq;
                use serde_json::Value;
                use std::fs::File;
                use std::io::BufReader;

                let file = File::open($filename).unwrap();
                let buf_reader = BufReader::new(file);
                let before: Value = serde_json::from_reader(buf_reader).unwrap();
                let object: $object = serde_json::from_value(before.clone()).unwrap();
                let after = serde_json::to_value(object).unwrap();
                assert_json_eq!(before, after);
            }
        };
    }
    pub(crate) use roundtrip;
}

// From https://github.com/rust-lang/cargo/issues/383#issuecomment-720873790,
// may they be forever blessed.
#[cfg(doctest)]
mod readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
