use regex::Regex;
use borsh::{BorshSerialize, BorshDeserialize};
use borsh::schema::{BorshSchema as BorshSchemaTrait, BorshSchemaContainer, Definition, Fields};
use borsh_derive::{BorshSchema};
use serde_derive::{Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
struct TypeInfo {
    name: Option<String>,
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
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, Clone)]
struct Person {
    name: String,
    cool: bool,
}

fn get_schema<T: BorshSchemaTrait>() -> BorshSchemaContainer {
    T::schema_container()
}

fn get_type(container: &BorshSchemaContainer, field_name: Option<&String>, declaration: &String) -> Type {
    //println!("Field name {:?}", field_name);
    //println!("Declaration {:?}", declaration);
    let mut name = match field_name {
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
                            let mut fields = Vec::new();
                            for field in v {
                                fields.push(get_type(container, Some(&field.0), &field.1));
                            }
                            if name == None {
                                name = Some(declaration.clone());
                            }
                            return Type::Struct(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
                        },
                        Fields::UnnamedFields(v) => {
                            let mut fields = Vec::new();
                            for field in v {
                                fields.push(get_type(container, None, &field));
                            }
                            return Type::EnumVariant(TypeInfo { name: name.clone(), length: Some(v.len() as u32), signed: None, fields: Some(fields) })
                        },
                        Fields::Empty => return Type::EnumVariant(TypeInfo { name: name.clone(), length: None, signed: None, fields: None }),
                    }
                },
                Definition::Array { elements: e, length: l } => {
                    let fields = vec![get_type(container, None, &e)];
                    return Type::Array(TypeInfo { name: name.clone(), length: Some(l.to_owned()), signed: None, fields: Some(fields) })
                },
                Definition::Sequence { elements: e } => {
                    let fields = vec![get_type(container, None, &e)];
                    return Type::Vec(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
                },
                Definition::Enum {variants: v} => {
                    let mut enums = Vec::new();
                    for ev in v {
                        enums.push(get_type(container, Some(&ev.0), &ev.1));
                    }
                    return Type::Enum(TypeInfo { name: name.clone(), length: Some(v.len() as u32), signed: None, fields: Some(enums) })
                },
                _ => {},
            }
        }
    }
    match declaration.as_str() {
        "bool" => return Type::Bool(TypeInfo {name: name.clone(), signed: None, length: None, fields: None}),
        "string" => return Type::String(TypeInfo {name: name.clone(), signed: None, length: None, fields: None}),
        _ => {},
    };
    let re_unsigned_int = Regex::new(r"^u(\d+)$").unwrap();
    match re_unsigned_int.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8 || bytes == 16) { panic!("Invalid unsigned integer bytes") }
            return Type::Integer(TypeInfo {name: name.clone(), signed: Some(false), length: Some(bytes), fields: None})
        },
        None => {},
    }
    let re_signed_int = Regex::new(r"^i(\d+)$").unwrap();
    match re_signed_int.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 1 || bytes == 2 || bytes == 4 || bytes == 8 || bytes == 16) { panic!("Invalid signed integer bytes") }
            return Type::Integer(TypeInfo {name: name.clone(), signed: Some(true), length: Some(bytes), fields: None})
        },
        None => {},
    }
    let re_float = Regex::new(r"^f(\d+)$").unwrap();
    match re_float.captures(declaration) {
        Some(bits_info) => {
            let bytes = bits_info.get(1).unwrap().as_str().parse::<u32>().unwrap().checked_div(8).unwrap();
            if !(bytes == 4 || bytes == 8) { panic!("Invalid signed integer bytes") }
            return Type::Float(TypeInfo {name: name.clone(), length: Some(bytes), signed: None, fields: None})
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
                        fields.push(get_type(container, None, &e));
                    }
                    return Type::Tuple(TypeInfo { name: name.clone(), length: Some(ve.len() as u32), signed: None, fields: Some(fields) })
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
                    let fields = vec![get_type(container, None, &e)];
                    return Type::Array(TypeInfo { name: name.clone(), length: Some(l.to_owned()), signed: None, fields: Some(fields) })
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
                    let fields = vec![get_type(container, None, &e)];
                    return Type::Vec(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
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
                    let fields = vec![get_type(container, None, &v[1].1)];
                    return Type::Option(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
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
                        get_type(container, None, &v[0].1), // Ok
                        get_type(container, None, &v[1].1), // Err
                    ];
                    return Type::Result(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
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
                    let fields = vec![get_type(container, None, &e)];
                    return Type::HashSet(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
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
                    let fields = vec![get_type(container, None, &e)];
                    return Type::HashMap(TypeInfo { name: name.clone(), length: None, signed: None, fields: Some(fields) })
                },
                _ => {}
            }
        },
        None => {},
    }
    Type::Undefined
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

    let ty = get_type(&ctr, Some(&ctr.declaration), &ctr.declaration);
    //println!("Type {:?}", ty);

    let tsch = TypeSchema { schema: ty };

    // Serialize it to a JSON string.
    let j = serde_json::to_string(&tsch).unwrap();

    // Print, write to a file, or send to an HTTP server.
    println!("{}", j);
}
