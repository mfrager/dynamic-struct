use uuid::Uuid;
use borsh::maybestd::io::Result;
use sophia::graph::{*, inmem::FastGraph};
use sophia::iri::Iri;
use sophia::ns::rdf;
use sophia::serializer::*;
use sophia::serializer::nt::NtSerializer;
use sophia::term::literal::convert::AsLiteral;
//use sophia::ns::{Namespace, rdfs, rdf};
//use sophia::parser::turtle;
//use sophia::term::Result as SophiaResult;
//use sophia::term::iri::error::InvalidIri;

pub mod schema;
use schema::*;

pub trait Build {
    fn build(&mut self, data: Option<&str>) -> Result<()>;
    fn path_element(&mut self, node: &Type, index: usize, root: bool) -> Result<String>;
    fn stack_push(&mut self, index: usize) -> Result<()>;
    fn stack_pop(&mut self) -> Result<()>;
    fn is_root(&self) -> bool;
    fn get_uri(&self, instance: bool) -> String;
}

pub struct Builder<'a> {
    schema: &'a TypeSchema,
    stack: Vec<(&'a Type, usize)>,
    path: Vec<String>,
    uri: Vec<String>,
    root: bool,
    pub graph: FastGraph,
}

impl<'a> Build for Builder<'a> {
    fn build(&mut self, data: Option<&str>) -> Result<()> {
        let top_index = self.stack.len() - 1;
        let node = self.stack[top_index];
        let mut container = false;
        let mut container_label: String = "".into();
        //println!("{:?}", self.path.join("/"));
        match node.0.datatype {
            DataType::Struct => {
                //println!("{:?}", node.0.datatype);
                container = true;
                container_label = format!("type/struct#{}", node.0.term.as_ref().unwrap().as_str()).to_string();
                self.uri.push(self.get_uri(false));
            },
            DataType::Tuple => {
                //println!("{:?}", node.0.datatype);
                container = true;
                container_label = "type/tuple".into();
                self.uri.push(self.get_uri(false));
            },
            DataType::Vec => {
                //println!("{:?}", node.0.datatype);
                container = true;
                container_label = "type/vec".into();
                self.uri.push(self.get_uri(false));
            },
            _ => {
                //println!("{:?}: {}", node.0.datatype, data.unwrap());
            }
        }
        //println!("{:?}", self.uri.join("|"));
        let prop = self.get_uri(true);
        if container {
            let base = "https://data.atellix.net";
            let class = format!("{}/{}", base, container_label.as_str());
            //println!("{:?} rdf:class {:?}", self.uri.last().unwrap(), class);
            self.graph.insert(&Iri::new(self.uri.last().unwrap()).unwrap(), &rdf::type_, &Iri::new(class.as_str()).unwrap()).unwrap();
            if self.uri.len() > 1 {
                let parent_index = self.uri.len() - 2;
                let parent = &self.uri[parent_index];
                //println!("{:?} {:?} {:?}", parent, prop, self.uri.last().unwrap());
                self.graph.insert(&Iri::new(parent.as_str()).unwrap(), &Iri::new(prop.as_str()).unwrap(), &Iri::new(self.uri.last().unwrap()).unwrap()).unwrap();
            }
        } else {
            //println!("{:?} {:?} {:?}", self.uri.last().unwrap(), prop, data.unwrap());
            self.graph.insert(&Iri::new(self.uri.last().unwrap()).unwrap(), &Iri::new(prop.as_str()).unwrap(), &data.unwrap().as_literal()).unwrap();
        }
        Ok(())
    }

    fn get_uri(&self, instance: bool) -> String {
        let base = "https://data.atellix.net";
        if instance {
            format!("{}/{}", base, self.path.join("/").as_str()).to_string()
        } else {
            let uuid = Uuid::new_v4();
            format!("{}/{}#{}", base, self.path.join("/").as_str(), uuid).to_string()
        }
    }

    fn is_root(&self) -> bool {
        self.root
    }

    fn stack_push(&mut self, index: usize) -> Result<()> {
        let pe;
        if self.root {
            self.root = false;
            self.stack.push((&self.schema.schema, 0));
            pe = self.path_element(&self.schema.schema, 0, true)?;
        } else {
            let top_index = self.stack.len() - 1;
            let top_node = self.stack[top_index];
            let field;
            if top_node.0.fields.is_none() && top_node.0.term.is_some() {
                let k = top_node.0.term.as_ref().unwrap();
                let node = self.schema.terms.get(k).unwrap();
                field = &node.fields.as_ref().unwrap()[index];
            } else {
                field = &top_node.0.fields.as_ref().unwrap()[index];
            }
            //println!("Push: {:?}", self.stack);
            self.stack.push((field, index));
            pe = self.path_element(field, index, false)?;
        }
        self.path.push(pe);
        Ok(())
    }

    fn stack_pop(&mut self) -> Result<()> {
        //println!("Pop: {:?}", self.stack);
        self.path.pop();
        let node = self.stack.pop().unwrap();
        match node.0.datatype {
            DataType::Struct => {
                self.uri.pop();
            },
            DataType::Tuple => {
                self.uri.pop();
            },
            DataType::Vec => {
                self.uri.pop();
            },
            _ => {}
        }
        Ok(())
    }

    fn path_element(&mut self, node: &Type, index: usize, root: bool) -> Result<String> {
        match node.datatype {
            DataType::Struct => {
                if root {
                    return Ok(node.term.as_ref().unwrap().to_string())
                } else {
                    if node.name.as_ref().is_some() {
                        return Ok(node.name.as_ref().unwrap().to_string())
                    } else {
                        return Ok(format!("{}", index.to_string()).to_string())
                    }
                }
            },
            _ => {
                if node.name.as_ref().is_some() {
                    return Ok(node.name.as_ref().unwrap().to_string())
                } else {
                    return Ok(format!("{}", index.to_string()).to_string())
                }
            },
        }
    }
}

pub trait CustomSerialize {
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()>;

    fn try_to_custom(&self, schema: &TypeSchema) -> Result<()> {
        let mut b = Builder {
            schema,
            stack: vec![],
            path: vec![],
            uri: vec![],
            root: true,
            graph: FastGraph::new(),
        };
        self.serialize(&mut b)?;
        let mut ntstr = NtSerializer::new_stringifier();
        let gr = ntstr.serialize_graph(&mut b.graph).unwrap().as_str();
        println!("{}", gr);
        Ok(())
    }

    fn push_node<B: Build>(&self, builder: &mut B, index: usize) -> Result<()> {
        builder.stack_push(index)?;
        Ok(())
    }

    fn pop_node<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.stack_pop()?;
        Ok(())
    }
}

impl CustomSerialize for u8 {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(Some(format!("{:?}", self).as_str()))?;
        Ok(())
    }
}

macro_rules! impl_for_integer {
    ($type: ident) => {
        impl CustomSerialize for $type {
            #[inline]
            fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
                builder.build(Some(format!("{:?}", self).as_str()))?;
                Ok(())
            }
        }
    };
}

impl_for_integer!(i8);
impl_for_integer!(i16);
impl_for_integer!(i32);
impl_for_integer!(i64);
impl_for_integer!(i128);
impl_for_integer!(u16);
impl_for_integer!(u32);
impl_for_integer!(u64);
impl_for_integer!(u128);

impl CustomSerialize for f32 {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(Some(format!("{:?}", self).as_str()))?;
        Ok(())
    }
}

impl CustomSerialize for f64 {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(Some(format!("{:?}", self).as_str()))?;
        Ok(())
    }
}

impl CustomSerialize for isize {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        CustomSerialize::serialize(&(*self as i64), builder)
    }
}

impl CustomSerialize for usize {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        CustomSerialize::serialize(&(*self as u64), builder)
    }
}

impl CustomSerialize for bool {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(Some(format!("{:?}", self).as_str()))?;
        Ok(())
    }
}

impl CustomSerialize for String {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(Some(self.as_str()))?;
        Ok(())
    }
}

impl<T> CustomSerialize for Vec<T>
where
    T: CustomSerialize,
{
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(Some(&self.len().to_string().as_str()))?;
        for rc in self {
            CustomSerialize::push_node(rc, builder, 0)?;
            CustomSerialize::serialize(rc, builder)?;
            CustomSerialize::pop_node(rc, builder)?;
        }
        Ok(())
    }
}

impl CustomSerialize for () {
    fn serialize<B: Build>(&self, _builder: &mut B) -> Result<()> {
        Ok(())
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $name:ident)+) => {
        impl<$($name),+> CustomSerialize for ($($name,)+)
        where $($name: CustomSerialize,)+
        {
            #[inline]
            fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
                builder.build(None)?;
                $(
                    self.$idx.push_node(builder, $idx)?;
                    self.$idx.serialize(builder)?;
                    self.$idx.pop_node(builder)?;
                )+
                Ok(())
            }
        }
    };
}

impl_tuple!(0 T0);
impl_tuple!(0 T0 1 T1);
impl_tuple!(0 T0 1 T1 2 T2);
impl_tuple!(0 T0 1 T1 2 T2 3 T3);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 19 T19);

