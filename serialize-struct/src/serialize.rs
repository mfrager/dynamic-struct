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
}

pub struct Builder<'a> {
    schema: &'a TypeSchema,
    stack: Vec<&'a Type>,
}

impl<'a> Build for Builder<'a> {
    fn build(&mut self, debug: Option<&str>) -> Result<()> {
        let top_index = self.stack.len() - 1;
        let node = self.stack[top_index];
        println!("Type: {:?}", node);
        match node.datatype {
            DataType::Struct => {},
            _ => {
                println!("{}", debug.unwrap());
            }
        }
        Ok(())
    }

    fn stack_push(&mut self, index: usize) -> Result<()> {
        let top_index = self.stack.len() - 1;
        let top_node = self.stack[top_index];
        let field = &top_node.fields.as_ref().unwrap()[index];
        self.stack.push(field);
        Ok(())
    }

    fn stack_pop(&mut self) -> Result<()> {
        self.stack.pop();
        Ok(())
    }
}

pub trait CustomSerialize {
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()>;

    fn try_to_custom(&self, schema: &TypeSchema) -> Result<()> {
        let mut b = Builder {
            schema,
            stack: vec![&schema.schema],
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
        builder.build(Some(format!("u8: {:?}", self).as_str()));
        Ok(())
    }
}

