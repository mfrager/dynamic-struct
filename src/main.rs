use regex::Regex;
use borsh::maybestd::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};
use borsh::schema::{BorshSchema as BorshSchemaTrait, BorshSchemaContainer, Definition, Fields};
use borsh_derive::{BorshSchema};
use serde_derive::{Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
struct TypeInfo {
    name: Option<String>,
    term: Option<String>,
    signed: Option<bool>,
    length: Option<u32>,
    fields: Option<Vec<Type>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
enum Type {
    Bool(TypeInfo),
    Integer(TypeInfo),
    Float(TypeInfo),
    String(TypeInfo),
    Enum(TypeInfo),
    EnumVariant(TypeInfo),
    Tuple(TypeInfo),
    Struct(TypeInfo),
    Array(TypeInfo),
    Vec(TypeInfo),
    Option(TypeInfo),
    Result(TypeInfo),
    HashSet(TypeInfo),
    HashMap(TypeInfo),
    Undefined,
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
    name: String,
    cool: bool,
    some: Something,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, Clone)]
struct Person {
    uuid: u128,
    name: String,
    cool: bool,
    other: Vec<Other>,
    friends: Vec<u128>,
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
                                return Type::Struct(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fs), term: Some(declaration.clone()) });
                            } else {
                                let found_struct = result.terms.get(&declaration.clone());
                                if found_struct == None {
                                    let mut fs = Vec::new();
                                    for field in v {
                                        fs.push(get_type(container, Some(&field.0), &field.1, result, false));
                                    }
                                    let ts = Type::Struct(TypeInfo { name: None, length: None, signed: None, fields: Some(fs), term: Some(declaration.clone()) });
                                    result.terms.insert(declaration.clone(), ts.clone());
                                }
                                return Type::Struct(TypeInfo { name: name.clone(), length: None, signed: None, fields: None, term: Some(declaration.clone()) });
                            }
                        },
                        Fields::UnnamedFields(v) => {
                            let mut fields = Vec::new();
                            for field in v {
                                fields.push(get_type(container, None, &field, result, false));
                            }
                            return Type::EnumVariant(TypeInfo { name: name.clone(), length: Some(v.len() as u32), signed: None, fields: Some(fields), term: None })
                        },
                        Fields::Empty => return Type::EnumVariant(TypeInfo { name: name.clone(), length: None, signed: None, fields: None, term: None }),
                    }
                },
                Definition::Array { elements: e, length: l } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type::Array(TypeInfo { name: name.clone(), length: Some(l.to_owned()), signed: None, fields: Some(fields), term: None })
                },
                Definition::Sequence { elements: e } => {
                    let fields = vec![get_type(container, None, &e, result, false)];
                    return Type::Vec(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields), term: None })
                },
                Definition::Enum {variants: v} => {
                    let found_enum = result.terms.get(&declaration.clone());
                    if found_enum == None {
                        let mut enums = Vec::new();
                        for ev in v {
                            enums.push(get_type(container, Some(&ev.0), &ev.1, result, false));
                        }
                        let ts = Type::Enum(TypeInfo { name: None, length: Some(v.len() as u32), signed: None, fields: Some(enums), term: Some(declaration.clone()) });
                        result.terms.insert(declaration.clone(), ts.clone());
                    }
                    return Type::Enum(TypeInfo { name: name.clone(), length: None, signed: None, fields: None, term: Some(declaration.clone()) })
                },
                _ => {},
            }
        }
    }
    match declaration.as_str() {
        "bool" => return Type::Bool(TypeInfo {name: name.clone(), signed: None, length: None, fields: None, term: None}),
        "string" => return Type::String(TypeInfo {name: name.clone(), signed: None, length: None, fields: None, term: None}),
        _ => {},
    };
    let re_unsigned_int = Regex::new(r"^u(\d+)$").unwrap();
    match re_unsigned_int.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8 || bytes == 16) { panic!("Invalid unsigned integer bytes") }
            return Type::Integer(TypeInfo {name: name.clone(), signed: Some(false), length: Some(bytes), fields: None, term: None})
        },
        None => {},
    }
    let re_signed_int = Regex::new(r"^i(\d+)$").unwrap();
    match re_signed_int.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8 || bytes == 16) { panic!("Invalid signed integer bytes") }
            return Type::Integer(TypeInfo {name: name.clone(), signed: Some(true), length: Some(bytes), fields: None, term: None})
        },
        None => {},
    }
    let re_float = Regex::new(r"^f(\d+)$").unwrap();
    match re_float.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 4 || bytes == 8) { panic!("Invalid signed integer bytes") }
            return Type::Float(TypeInfo {name: name.clone(), length: Some(bytes), signed: None, fields: None, term: None})
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
                    return Type::Tuple(TypeInfo { name: name.clone(), length: Some(ve.len() as u32), signed: None, fields: Some(fields), term: None })
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
                    return Type::Array(TypeInfo { name: name.clone(), length: Some(l.to_owned()), signed: None, fields: Some(fields), term: None })
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
                    return Type::Vec(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields), term: None })
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
                    return Type::Option(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields), term: None })
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
                    return Type::Result(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields), term: None })
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
                    return Type::HashSet(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields), term: None })
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
                    return Type::HashMap(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields), term: None })
                },
                _ => {}
            }
        },
        None => {},
    }
    Type::Undefined
}

struct TypeIterator<'a> {
    schema: &'a TypeSchema,
    stack: Vec<(&'a Type, &'a Type)>,
}

impl<'a> TypeIterator<'a> {
    fn new(schema: &'a TypeSchema) -> TypeIterator<'a> {
        TypeIterator { stack: vec![(&Type::Undefined, &schema.schema)], schema: schema }
    }

    fn add_child_nodes(&mut self, tinfo: &'a TypeInfo, lookup: bool, schema: &'a TypeSchema, parent: &'a Type) {
        if tinfo.fields.is_some() {
            let subfields: &Vec<Type> = &tinfo.fields.as_ref().unwrap().as_ref();
            for child in subfields.iter().rev() {
                self.stack.push((parent, child));
            }
        } else if lookup {
            let rnode = schema.terms.get(&tinfo.term.clone().unwrap());
            if rnode.is_some() {
                self.add_nodes(&rnode.unwrap(), schema);
            }
        }
    }

    fn add_nodes(&mut self, node: &'a Type, schema: &'a TypeSchema) {
        match node {
            Type::Bool(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Integer(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Float(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::String(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Enum(ti) => self.add_child_nodes(&ti, true, schema, node),
            Type::EnumVariant(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Tuple(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Struct(ti) => self.add_child_nodes(&ti, true, schema, node),
            Type::Array(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Vec(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Option(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::Result(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::HashSet(ti) => self.add_child_nodes(&ti, false, schema, node),
            Type::HashMap(ti) => self.add_child_nodes(&ti, false, schema, node),
            _ => {}
        }
    }
}

impl<'a> Iterator for TypeIterator<'a> {
    type Item = (&'a Type, &'a Type);

    fn next(&mut self) -> Option<Self::Item> {
        match self.stack.pop() {
            None => None,
            Some((par, node)) => {
                self.add_nodes(&node, &self.schema);
                Some((par, node))
            }
        }
    }
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

    let mut tsch = TypeSchema { schema: Type::Undefined, terms: HashMap::new() };
    tsch.schema = get_type(&ctr, Some(&ctr.declaration), &ctr.declaration, &mut tsch, true);
    //println!("Type {:?}", ty);

    // Serialize it to a JSON string.
    //let j = serde_json::to_string(&tsch).unwrap();

    // Print, write to a file, or send to an HTTP server.
    //println!("{}", j);

    let mut iter = TypeIterator::new(&tsch);
    let mut counter: u32 = 0;
    while let Some(node) = iter.next() {
        counter += 1;
        println!("{:?}", counter);
        println!("Item {:?}", node.1);
        println!("Parent {:?}", node.0);
        println!("");
    }
}
