use core::marker::PhantomData;
use regex::Regex;
use borsh::maybestd::collections::HashMap;
use borsh::schema::{BorshSchema as BorshSchemaTrait, BorshSchemaContainer, Definition, Fields};
use serde_derive::{Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[repr(u8)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    Enum,
    Variant,
    Tuple,
    Struct,
    Array,
    Vec,
    Option,
    Result,
    HashSet,
    HashMap,
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Type {
    datatype: DataType,
    name: Option<String>,
    term: Option<String>,
    signed: Option<bool>,
    length: Option<u32>,
    fields: Option<Vec<Type>>,
}

impl Default for Type {
    fn default() -> Self {
        Type {
            datatype: DataType::Undefined,
            name: None,
            term: None,
            signed: None,
            length: None,
            fields: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeSchema {
    schema: Type,
    terms: HashMap<String, Type>,
}

pub fn get_schema<T: BorshSchemaTrait>() -> TypeSchema {
    let ctr = T::schema_container();
    let mut tsch = TypeSchema { schema: Type::default(), terms: HashMap::new() };
    tsch.schema = get_type(&ctr, Some(&ctr.declaration), &ctr.declaration, &mut tsch, true);
    tsch
}

pub fn get_type(container: &BorshSchemaContainer, field_name: Option<&String>, declaration: &String, result: &mut TypeSchema, root: bool) -> Type {
    let name = match field_name {
        Some(str_ref) => { Some(str_ref.to_owned()) },
        None => None,
    };
    if !(
        declaration.starts_with("HashSet<") || declaration.starts_with("HashMap<") ||
        declaration.starts_with("Option<") || declaration.starts_with("Result<")
    ) {
        let definition = container.definitions.get(declaration);
        if definition.is_some() {
            match definition.unwrap() {
                Definition::Struct {fields: f} => {
                    match f {
                        Fields::NamedFields(v) => {
                            if root {
                                let mut fs = Vec::new();
                                for field in v {
                                    fs.push(get_type(container, Some(&field.0), &field.1, result, false));
                                }
                                return Type { datatype: DataType::Struct, name: name.clone(), length: None, signed: None, fields: Some(fs), term: Some(declaration.clone()) };
                            } else {
                                let found_struct = result.terms.get(&declaration.clone());
                                if found_struct == None {
                                    let mut fs = Vec::new();
                                    for field in v {
                                        fs.push(get_type(container, Some(&field.0), &field.1, result, false));
                                    }
                                    let ts = Type { datatype: DataType::Struct, name: None, length: None, signed: None, fields: Some(fs), term: Some(declaration.clone()) };
                                    result.terms.insert(declaration.clone(), ts.clone());
                                }
                                return Type { datatype: DataType::Struct, name: name.clone(), length: None, signed: None, fields: None, term: Some(declaration.clone()) };
                            }
                        },
                        Fields::UnnamedFields(v) => {
                            let mut fields = Vec::new();
                            for field in v {
                                fields.push(get_type(container, None, &field, result, false));
                            }
                            return Type {datatype: DataType::Variant, name: name.clone(), length: Some(v.len() as u32), signed: None, fields: Some(fields), term: None };
                        },
                        Fields::Empty => return Type { datatype: DataType::Variant, name: name.clone(), length: None, signed: None, fields: None, term: None },
                    }
                },
                Definition::Array { elements: e, length: l } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type {datatype: DataType::Array, name: name.clone(), length: Some(l.to_owned()), signed: None, fields: Some(fields), term: None }
                },
                Definition::Sequence { elements: e } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type {datatype: DataType::Vec, name: name.clone(), length: None, signed: None, fields: Some(fields), term: None }
                },
                Definition::Enum {variants: v} => {
                    let found_enum = result.terms.get(&declaration.clone());
                    if found_enum == None {
                        let mut enums = Vec::new();
                        for ev in v {
                            enums.push(get_type(container, Some(&ev.0), &ev.1, result, false));
                        }
                        let ts = Type {datatype: DataType::Enum, name: None, length: Some(v.len() as u32), signed: None, fields: Some(enums), term: Some(declaration.clone()) };
                        result.terms.insert(declaration.clone(), ts.clone());
                    }
                    return Type {datatype: DataType::Enum, name: name.clone(), length: None, signed: None, fields: None, term: Some(declaration.clone()) }
                },
                _ => {},
            }
        }
    }
    match declaration.as_str() {
        "bool" => return Type { datatype: DataType::Bool, name: name.clone(), signed: None, length: None, fields: None, term: None},
        "string" => return Type { datatype: DataType::String, name: name.clone(), signed: None, length: None, fields: None, term: None},
        _ => {},
    };
    let re_unsigned_int = Regex::new(r"^u(\d+)$").unwrap();
    match re_unsigned_int.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8 || bytes == 16) { panic!("Invalid unsigned integer bytes") }
            return Type {datatype: DataType::Int, name: name.clone(), signed: Some(false), length: Some(bytes), fields: None, term: None}
        },
        None => {},
    }
    let re_signed_int = Regex::new(r"^i(\d+)$").unwrap();
    match re_signed_int.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8 || bytes == 16) { panic!("Invalid signed integer bytes") }
            return Type {datatype: DataType::Int, name: name.clone(), signed: Some(true), length: Some(bytes), fields: None, term: None}
        },
        None => {},
    }
    let re_float = Regex::new(r"^f(\d+)$").unwrap();
    match re_float.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 4 || bytes == 8) { panic!("Invalid signed integer bytes") }
            return Type {datatype: DataType::Float, name: name.clone(), length: Some(bytes), signed: None, fields: None, term: None}
        },
        None => {},
    }
    let re_tuple = Regex::new(r"^Tuple<.*>$").unwrap();
    match re_tuple.captures(declaration) {
        Some(tuple_txt) => {
            let tuple_def = container.definitions.get(tuple_txt.get(0).unwrap().as_str()).unwrap();
            match tuple_def {
                Definition::Tuple { elements: ve } => {
                    let mut fields = Vec::new();
                    for e in ve {
                        fields.push(get_type(container, None, &e, result, false));
                    }
                    return Type {datatype: DataType::Tuple, name: name.clone(), length: Some(ve.len() as u32), signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    let re_array = Regex::new(r"^Array<.*>$").unwrap();
    match re_array.captures(declaration) {
        Some(array_txt) => {
            let array_def = container.definitions.get(array_txt.get(0).unwrap().as_str()).unwrap();
            match array_def {
                Definition::Array { elements: e, length: l } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type {datatype: DataType::Array, name: name.clone(), length: Some(l.to_owned()), signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    let re_vec = Regex::new(r"^Vec<.*>$").unwrap();
    match re_vec.captures(declaration) {
        Some(vec_txt) => {
            let vec_def = container.definitions.get(vec_txt.get(0).unwrap().as_str()).unwrap();
            match vec_def {
                Definition::Sequence { elements: e } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type {datatype: DataType::Vec, name: name.clone(), length: None, signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    let re_option = Regex::new(r"^Option<.*>$").unwrap();
    match re_option.captures(declaration) {
        Some(option_txt) => {
            let option_def = container.definitions.get(option_txt.get(0).unwrap().as_str()).unwrap();
            match option_def {
                Definition::Enum { variants: v } => {
                    let fields = vec![get_type(container, None, &v[1].1, result, false)];
                    return Type {datatype: DataType::Option, name: name.clone(), length: None, signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    let re_result = Regex::new(r"^Result<.*>$").unwrap();
    match re_result.captures(declaration) {
        Some(result_txt) => {
            let result_def = container.definitions.get(result_txt.get(0).unwrap().as_str()).unwrap();
            match result_def {
                Definition::Enum { variants: v } => {
                    let fields = vec![
                        get_type(container, None, &v[0].1, result, false), // Ok
                        get_type(container, None, &v[1].1, result, false), // Err
                    ];
                    return Type {datatype: DataType::Result, name: name.clone(), length: None, signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    let re_hashset = Regex::new(r"^HashSet<.*>$").unwrap();
    match re_hashset.captures(declaration) {
        Some(hashset_txt) => {
            let hashset_def = container.definitions.get(hashset_txt.get(0).unwrap().as_str()).unwrap();
            match hashset_def {
                Definition::Sequence { elements: e } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type {datatype: DataType::HashSet, name: name.clone(), length: None, signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    let re_hashmap = Regex::new(r"^HashMap<.*>$").unwrap();
    match re_hashmap.captures(declaration) {
        Some(hashmap_txt) => {
            let hashmap_def = container.definitions.get(hashmap_txt.get(0).unwrap().as_str()).unwrap();
            match hashmap_def {
                Definition::Sequence { elements: e } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type {datatype: DataType::HashMap, name: name.clone(), length: None, signed: None, fields: Some(fields), term: None }
                },
                _ => {}
            }
        },
        None => {},
    }
    Type::default()
}

pub struct TypeIterator<'a, T> {
    schema: &'a TypeSchema,
    stack: Vec<(Option<&'a Type>, &'a Type)>,
    data: PhantomData<&'a T>,
}

impl<'a, T: BorshSchemaTrait> TypeIterator<'a, T> {
    pub fn new(schema: &'a TypeSchema) -> TypeIterator<'a, T> {
        TypeIterator { stack: vec![(None, &schema.schema)], schema: schema, data: PhantomData {} }
    }

    fn add_child_nodes(&mut self, node: &'a Type, lookup: bool, schema: &'a TypeSchema) {
        if node.fields.is_some() {
            let subfields: &Vec<Type> = &node.fields.as_ref().unwrap().as_ref();
            for child in subfields.iter().rev() {
                self.stack.push((Some(node), child));
            }
        } else if lookup {
            let rnode = schema.terms.get(&node.term.clone().unwrap());
            if rnode.is_some() {
                self.add_nodes(&rnode.unwrap(), schema);
            }
        }
    }

    fn add_nodes(&mut self, node: &'a Type, schema: &'a TypeSchema) {
        match node.datatype {
            DataType::Undefined => {},
            DataType::Struct => self.add_child_nodes(node, true, schema),
            DataType::Enum => self.add_child_nodes(node, true, schema),
            _ => self.add_child_nodes(node, false, schema),
        }
    }
}

impl<'a, T: BorshSchemaTrait> Iterator for TypeIterator<'a, T> {
    type Item = (Option<&'a Type>, &'a Type);

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            None => None,
            Some((parent, node)) => {
                self.add_nodes(&node, &self.schema);
                Some((parent, node))
            }
        }
    }
}

