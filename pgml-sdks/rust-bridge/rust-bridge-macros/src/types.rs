use quote::ToTokens;
use std::boxed::Box;
use std::string::ToString;
use syn::visit::{self, Visit};

#[derive(Debug, Clone)]
pub struct ReferenceType {
    pub ty: Box<SupportedType>,
    pub mutable: bool,
}

impl ReferenceType {
    pub fn new(ty: SupportedType, mutable: bool) -> Self {
        Self {
            ty: Box::new(ty),
            mutable,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum SupportedType {
    Reference(ReferenceType),
    str,
    String,
    bool,
    Vec(Box<SupportedType>),
    HashMap((Box<SupportedType>, Box<SupportedType>)),
    Option(Box<SupportedType>),
    Tuple(Vec<SupportedType>),
    S, // Self for return types only
    i64,
    u64,
    i32,
    f64,
    CustomType(String),
}

impl std::fmt::Display for SupportedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_language_string(&None))
    }
}

impl SupportedType {
    pub fn to_type(&self, language: Option<&str>) -> syn::Result<syn::Type> {
        syn::parse_str(&self.to_language_string(&language))
    }

    pub fn to_language_string(&self, language: &Option<&str>) -> String {
        match self {
            SupportedType::Reference(t) => {
                if t.mutable {
                    format!("&mut {}", t.ty.to_language_string(language))
                } else {
                    format!("&{}", t.ty.to_language_string(language))
                }
            }
            SupportedType::str => "str".to_string(),
            SupportedType::String => "String".to_string(),
            SupportedType::bool => "bool".to_string(),
            SupportedType::Vec(v) => format!("Vec<{}>", v.to_language_string(language)),
            SupportedType::HashMap((k, v)) => {
                format!(
                    "HashMap<{},{}>",
                    k.to_language_string(language),
                    v.to_language_string(language)
                )
            }
            SupportedType::Tuple(t) => {
                let mut types = Vec::new();
                for ty in t {
                    types.push(ty.to_language_string(language));
                }
                format!("({})", types.join(","))
            }
            SupportedType::S => "Self".to_string(),
            SupportedType::Option(v) => format!("Option<{}>", v.to_language_string(language)),
            SupportedType::i64 => "i64".to_string(),
            SupportedType::u64 => "u64".to_string(),
            SupportedType::i32 => "i32".to_string(),
            SupportedType::f64 => "f64".to_string(),
            // Our own types
            SupportedType::CustomType(t) => format!("{}{}", t, language.unwrap_or("")),
        }
    }
}

#[derive(Default)]
pub struct GetSupportedType {
    pub ty: Option<SupportedType>,
}

impl GetSupportedType {
    pub fn get_type(i: &syn::Type) -> SupportedType {
        let mut s = Self::default();
        s.visit_type(i);
        s.ty.expect("Error getting type from Type")
    }

    pub fn get_type_from_path(i: &syn::TypePath) -> SupportedType {
        let mut s = Self::default();
        s.visit_path_segment(i.path.segments.last().expect("No path segment found"));
        s.ty.expect("Error getting type from TypePath")
    }

    pub fn get_type_from_path_argument(i: &syn::PathArguments) -> SupportedType {
        let mut s = Self::default();
        s.visit_path_arguments(i);
        s.ty.expect("Error getting type from PathArguments")
    }

    pub fn get_type_from_generic_argument(i: &syn::GenericArgument) -> SupportedType {
        let mut s = Self::default();
        s.visit_generic_argument(i);
        s.ty.expect("Error gettingtype from GenericArgument")
    }

    pub fn get_type_from_angle_bracketed_generic_arguments(
        i: &syn::AngleBracketedGenericArguments,
    ) -> SupportedType {
        let mut s = Self::default();
        s.visit_angle_bracketed_generic_arguments(i);
        s.ty.expect("Error getting type from AngleBracketedGenericArguments")
    }
}

impl<'ast> Visit<'ast> for GetSupportedType {
    fn visit_type(&mut self, i: &syn::Type) {
        self.ty = Some(match i {
            syn::Type::Reference(r) => SupportedType::Reference(ReferenceType::new(
                Self::get_type(&r.elem),
                r.mutability.is_some(),
            )),
            syn::Type::Path(p) => Self::get_type_from_path(p),
            syn::Type::Tuple(t) => {
                let values: Vec<SupportedType> = t
                    .elems
                    .pairs()
                    .map(|p| {
                        let ty = p.value();
                        Self::get_type(ty)
                    })
                    .collect();
                SupportedType::Tuple(values)
            }
            _ => panic!(
                "Type is not supported yet {:?}",
                i.to_token_stream().to_string()
            ),
        });
    }

    fn visit_path_segment(&mut self, i: &syn::PathSegment) {
        let segment_name = i.ident.to_string();
        self.ty = match segment_name.as_str() {
            "str" => Some(SupportedType::str),
            "String" => Some(SupportedType::String),
            "bool" => Some(SupportedType::bool),
            "Vec" => Some(SupportedType::Vec(Box::new(
                Self::get_type_from_path_argument(&i.arguments),
            ))),
            "Option" => Some(SupportedType::Option(Box::new(
                Self::get_type_from_path_argument(&i.arguments),
            ))),
            "HashMap" => match &i.arguments {
                syn::PathArguments::AngleBracketed(a) => {
                    let mut p = a.args.clone().into_pairs();
                    let key = p.next().unwrap().value().to_owned();
                    let value = p.next().unwrap().value().to_owned();
                    Some(SupportedType::HashMap((
                        Box::new(Self::get_type_from_generic_argument(&key)),
                        Box::new(Self::get_type_from_generic_argument(&value)),
                    )))
                }
                _ => panic!(
                    "Type is not supported yet {:?}",
                    i.to_token_stream().to_string()
                ),
            },
            "Self" => Some(SupportedType::S),
            "i64" => Some(SupportedType::i64),
            "u64" => Some(SupportedType::u64),
            "i32" => Some(SupportedType::i32),
            "f64" => Some(SupportedType::f64),
            // Our own types
            t => Some(SupportedType::CustomType(t.to_string())),
        };

        // println!("SELF TYPE {:?}", self.ty);

        if self.ty.is_none() {
            visit::visit_path_segment(self, i);
        }
    }
}

#[derive(Debug)]
pub enum OutputType {
    Result(SupportedType),
    Default,
    Other(SupportedType), // Other(String),
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Default)]
pub struct GetOutputType {
    pub output: OutputType,
}

impl<'ast> Visit<'ast> for GetOutputType {
    fn visit_type(&mut self, i: &syn::Type) {
        visit::visit_type(self, i);
    }

    fn visit_path_segment(&mut self, i: &syn::PathSegment) {
        let segment_name = i.ident.to_string();
        if segment_name == "Result" {
            if let syn::PathArguments::AngleBracketed(args) = &i.arguments {
                let ty = GetSupportedType::get_type_from_angle_bracketed_generic_arguments(args);
                self.output = OutputType::Result(ty);
            } else {
                panic!(
                    "Error getting OutputType. Requested PathArgument is currently not supported"
                );
            }
        } else {
            let mut get_supported_type = GetSupportedType::default();
            get_supported_type.visit_path_segment(i);
            match get_supported_type.ty {
                Some(ty) => {
                    self.output = OutputType::Other(ty);
                }
                None => visit::visit_path_segment(self, i),
            };
        }
    }
}
