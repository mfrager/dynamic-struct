use std::collections::HashSet;
use uuid::Uuid;
use regex::Regex;
use borsh::maybestd::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};
use borsh::schema::{BorshSchema as BorshSchemaTrait, BorshSchemaContainer, Definition, Fields};
use borsh_derive::{BorshSchema};
use serde_derive::{Serialize};
use sophia::graph::{*, inmem::FastGraph};
use sophia::iri::Iri;
use sophia::ns::{Namespace, rdfs, rdf};
use sophia::parser::turtle;
use sophia::serializer::*;
use sophia::serializer::nt::NtSerializer;
use sophia::term::Result;
use sophia::term::iri::error::InvalidIri;
use sophia::term::literal::convert::AsLiteral;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[repr(u8)]
enum DataType {
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
struct Type {
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
struct TypeSchema {
    schema: Type,
    terms: HashMap<String, Type>,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, Clone)]
enum Something {
    A,
    B,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, Clone)]
struct Other {
    label: String,
    cool: bool,
    some: Something,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, Clone)]
struct Person {
    name: String,
    age: u32,
    zoom: Other,
    thing: Vec<Other>,
}

fn get_schema<T: BorshSchemaTrait>() -> BorshSchemaContainer {
    T::schema_container()
}

fn get_type(container: &BorshSchemaContainer, field_name: Option<&String>, declaration: &String, result: &mut TypeSchema, root: bool) -> Type {
    //println!("Field name {:?}", field_name);
    //println!("Declaration {:?}", declaration);
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

struct TypeIterator<'a> {
    schema: &'a TypeSchema,
    stack: Vec<(&'a Type, String)>,
    seen: HashSet<String>,
}

impl<'a> TypeIterator<'a> {
    fn new(schema: &'a TypeSchema) -> TypeIterator<'a> {
        TypeIterator { stack: vec![(&schema.schema, String::new())], schema: schema, seen: HashSet::new() }
    }

    fn add_child_nodes(&mut self, node: &'a Type, lookup: bool, schema: &'a TypeSchema) {
        if node.fields.is_some() {
            let subfields: &Vec<Type> = &node.fields.as_ref().unwrap().as_ref();
            for child in subfields.iter().rev() {
                self.stack.push((child, String::new()));
            }
        } else if lookup {
            let term = node.term.to_owned().unwrap();
            if !self.seen.get(&term).is_some() {
                self.seen.insert(term);
                let rnode = schema.terms.get(&node.term.clone().unwrap());
                if rnode.is_some() {
                    self.add_nodes(&rnode.unwrap(), schema);
                }
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

impl<'a> Iterator for TypeIterator<'a> {
    type Item = (&'a Type, String);

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            None => None,
            Some((node, path)) => {
                self.add_nodes(&node, &self.schema);
                Some((node, path))
            }
        }
    }
}

fn build_rdf_instance(root: &Type) -> Result<FastGraph> {
    let ex = Namespace::new("http://example.org/").unwrap();
    //let foaf = Namespace::new("http://xmlns.com/foaf/0.1/").unwrap();
    let mut graph: FastGraph = FastGraph::new();
    let term = root.term.to_owned();
    let name = root.name.to_owned();
    let binding;
    if term.is_some() {
        binding = root.term.to_owned().unwrap();
    } else if name.is_some() {
        binding = root.name.to_owned().unwrap();
    } else {
        return Ok(graph)
    }
    //println!("{:?}", binding);
    let base_uri = ex.get(binding.as_str()).unwrap();
    match root.datatype {
        DataType::Struct => {
            graph.insert(&base_uri, &rdf::type_, &rdfs::Class)?;
        },
        _ => {
            graph.insert(&base_uri, &rdf::type_, &rdf::Property)?;
        },
        //DataType::Int(ti) => {},
        //DataType::Struct(ti) => {
        //DataType::Vec(ti) => {},
        //DataType::Enum(ti) => {},
        //DataType::Variant(ti) => {},
        //DataType::Bool(ti) => {},
        //DataType::Float(ti) => {},
        //DataType::Tuple(ti) => {},
        //DataType::Array(ti) => {},
        //DataType::Option(ti) => {},
        //DataType::Result(ti) => {},
        //DataType::HashSet(ti) => {},
        //DataType::HashMap(ti) => {},
    }
    Ok(graph)
}

fn main() {
    //let person = Person { name: "Alice".into(), age: 30 };
    
    // Serialize the person struct to bytes
    //let encoded = person.try_to_vec().unwrap();
    
    // Deserialize the bytes back into a person struct
    //let decoded = Person::try_from_slice(&encoded).unwrap();
    //println!("{:?}", decoded); // Output: Person { name: "Alice", age: 30 }

    let ctr = get_schema::<Person>();
    //println!("Schema container {:?}", ctr);

    //let def = ctr.definitions.get(&ctr.declaration).unwrap();
    //println!("Definition {:?}", def);

    let mut tsch = TypeSchema { schema: Type::default(), terms: HashMap::new() };
    tsch.schema = get_type(&ctr, Some(&ctr.declaration), &ctr.declaration, &mut tsch, true);
    //println!("Type {:?}", ty);

    // Serialize it to a JSON string.
    let j = serde_json::to_string(&tsch).unwrap();

    // Print, write to a file, or send to an HTTP server.
    println!("{}", j);

    let mut graph: FastGraph = FastGraph::new();
    let mut iter = TypeIterator::new(&tsch);
    let mut counter: u32 = 0;
    let mut seen: HashSet<String> = HashSet::new();
    while let Some(node) = iter.next() {
        counter += 1;
        println!("{:?}", counter);
        println!("Item {:?}", node.0);
        //println!("Parent {:?}", node.0);
        if match node.0.datatype {
            DataType::Struct => {
                let term = node.0.term.to_owned().unwrap();
                if seen.get(&term).is_some() { false } else { seen.insert(term); true }
            },
            DataType::Enum => {
                let term = node.0.term.to_owned().unwrap();
                if seen.get(&term).is_some() { false } else { seen.insert(term); true }
            },
            _ => true,
        } {
            graph.insert_all(build_rdf_instance(&node.0).unwrap().triples()).unwrap();
        }
        println!("");
    }
    let mut nt_stringifier = NtSerializer::new_stringifier();
    let example2 = nt_stringifier.serialize_graph(&mut graph).unwrap().as_str();
    println!("The resulting graph\n{}", example2);
}
