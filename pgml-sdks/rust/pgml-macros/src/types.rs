use quote::ToTokens;
use std::boxed::Box;
use std::string::ToString;
use syn::visit::{self, Visit};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum SupportedType {
    Reference(Box<SupportedType>),
    str,
    String,
    Vec(Box<SupportedType>),
    HashMap((Box<SupportedType>, Box<SupportedType>)),
    Option(Box<SupportedType>),
    Json(Box<SupportedType>),
    DateTime(Box<SupportedType>),
    Tuple(Vec<SupportedType>),
    S, // Self for return types only
    Utc,
    i64,
    f64,
    // Our own types
    Database,
    Collection,
    Splitter,
    Model,
}

impl ToString for SupportedType {
    fn to_string(&self) -> String {
        match self {
            SupportedType::Reference(t) => format!("&{}", t.to_string()),
            SupportedType::str => "str".to_string(),
            SupportedType::String => "String".to_string(),
            SupportedType::Vec(v) => format!("Vec<{}>", v.to_string()),
            SupportedType::HashMap((k, v)) => {
                format!("HashMap<{},{}>", k.to_string(), v.to_string())
            }
            SupportedType::Tuple(v) => {
                let mut output = String::new();
                v.iter().for_each(|ty| {
                    output.push_str(&format!("{},", ty.to_string()));
                });
                format!("({})", output)
            }
            SupportedType::S => "Self".to_string(),
            SupportedType::Option(v) => format!("Option<{}>", v.to_string()),
            SupportedType::i64 => "i64".to_string(),
            SupportedType::f64 => "f64".to_string(),
            SupportedType::Json(v) => format!("Json<{}>", v.to_string()),
            SupportedType::DateTime(v) => format!("DateTime<{}>", v.to_string()),
            SupportedType::Utc => "Utc".to_string(),
            // Our own types
            SupportedType::Database => "Database".to_string(),
            SupportedType::Collection => "Collection".to_string(),
            SupportedType::Splitter => "Splitter".to_string(),
            SupportedType::Model => "Model".to_string(),
        }
    }
}

impl SupportedType {
    pub fn to_type(&self) -> syn::Result<syn::Type> {
        syn::parse_str(&self.to_string())
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
        // println!("THE PATH {:?}", i);
        s.visit_path(&i.path);
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
            syn::Type::Reference(r) => SupportedType::Reference(Box::new(Self::get_type(&r.elem))),
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
            "Json" => Some(SupportedType::Json(Box::new(
                Self::get_type_from_path_argument(&i.arguments),
            ))),
            "DateTime" => Some(SupportedType::DateTime(Box::new(
                Self::get_type_from_path_argument(&i.arguments),
            ))),
            "Self" => Some(SupportedType::S),
            "Utc" => Some(SupportedType::Utc),
            "i64" => Some(SupportedType::i64),
            "f64" => Some(SupportedType::f64),
            // Our own types
            "Database" => Some(SupportedType::Database),
            "Collection" => Some(SupportedType::Collection),
            "Splitter" => Some(SupportedType::Splitter),
            "Model" => Some(SupportedType::Model),
            _ => None,
        };

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
