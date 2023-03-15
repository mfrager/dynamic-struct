use core::fmt::Debug;
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
    fn build(&mut self, debug: &str);
}

pub struct Builder {}

impl Build for Builder {
    fn build(&mut self, debug: &str) {
        println!("{}", debug);
    }
}

pub trait CustomSerialize {
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()>;

    fn try_to_custom(&self) -> Result<()> {
        let mut b = Builder {};
        self.serialize(&mut b)?;
        Ok(())
    }
}

impl CustomSerialize for u8 {
    #[inline]
    fn serialize<B: Build>(&self, builder: &mut B) -> Result<()> {
        builder.build(format!("u8: {:?}", self).as_str());
        Ok(())
    }
}

