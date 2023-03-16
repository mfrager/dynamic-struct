use borsh::maybestd::{
    //borrow::{Cow, ToOwned},
    //boxed::Box,
    //collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    //io::{ErrorKind, Result, Write},
    io::Result,
    //string::String,
    //vec::Vec,
};

pub mod schema;
use schema::*;

pub trait Build {
    fn build(&mut self, debug: Option<&str>) -> Result<()>;
    fn stack_push(&mut self, index: usize) -> Result<()>;
    fn stack_pop(&mut self) -> Result<()>;
    fn is_root(&self) -> bool;
    fn path_element(&mut self, node: &Type, index: usize, root: bool) -> Result<String>;
}

pub struct Builder<'a> {
    schema: &'a TypeSchema,
    stack: Vec<(&'a Type, usize)>,
    path: Vec<String>,
    root: bool,
}

impl<'a> Build for Builder<'a> {
    fn build(&mut self, debug: Option<&str>) -> Result<()> {
        let top_index = self.stack.len() - 1;
        let node = self.stack[top_index];
        println!("{:?}", self.path.join("/"));
        match node.0.datatype {
            DataType::Struct => {
                println!("{:?}", node.0.datatype);
            },
            DataType::Tuple => {
                println!("{:?}", node.0.datatype);
            },
            _ => {
                println!("{:?}: {}", node.0.datatype, debug.unwrap());
            }
        }
        Ok(())
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
        self.stack.pop();
        self.path.pop();
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
            root: true,
        };
        self.serialize(&mut b)?;
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

