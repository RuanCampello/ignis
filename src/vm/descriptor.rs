//! JVM Type Descriptor
//!
//! This module provides types and utilities for parsing, representing, and serializing
//! Java field descriptors as defined in the Java Virtual Machine Specification [4.3.2]
//!
//! A **field descriptor** is a string that encodes the type of a Java field (or method parameter)
//! in bytecode format. It is stored in the class file's field_info structures and used throughout
//! the JVM for type checking, method resolution, and reflection.
//!
//! ## Descriptor Format
//!
//! Field descriptors consist of one of the following:
//!
//! ### Primitive Types
//! ```text
//! ╭──────┬───────────┬───────────╮
//! │ Char │ Type      │ Java Type │
//! ├──────┼───────────┼───────────┤
//! │ B    │ Byte      │ byte      │
//! │ C    │ Char      │ char      │
//! │ D    │ Double    │ double    │
//! │ F    │ Float     │ float     │
//! │ I    │ Integer   │ int       │
//! │ J    │ Long      │ long      │
//! │ S    │ Short     │ short     │
//! │ Z    │ Boolean   │ boolean   │
//! ╰──────┴───────────┴───────────╯
//! ```
//! ### Reference Types
//!
//! ```text
//! ╭──────────────────┬──────────────────────┬─────────────╮
//! │ Format           │ Example              │ Java Type   │
//! ├──────────────────┼──────────────────────┼─────────────┤
//! │ L<class_name>;   │ Ljava/lang/String;   │ String      │
//! │ [<descriptor>    │ [I                   │ int[]       │
//! │ [<descriptor>    │ [[Ljava/lang/String; │ String[][]  │
//! ╰──────────────────┴──────────────────────┴─────────────╯
//! ```
//!
//! ### Array Types
//!
//! Arrays are represented by prefixing the base type descriptor with `[` for each dimension:
//!
//! ```text
//! int         -> I
//! int[]       -> [I
//! int[][]     -> [[I
//! int[][][]   -> [[[I
//! String[][]  -> [[Ljava/lang/String;
//! ```
//! [4.3.2](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.3.2)
//! [4.3.3](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.3.3)
//! [4.4](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.4)

use std::str::{Chars, FromStr};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(in crate::vm) struct MethodDescriptor {
    parameters: Vec<TypeDescriptor>,
    return_type: TypeDescriptor,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(in crate::vm) enum TypeDescriptor {
    Byte,
    Char,
    Double,
    Float,
    Integer,
    Long,
    Short,
    Boolean,
    Void,
    Array(Box<TypeDescriptor>, u8),
    Object(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected end of input")]
    UnexpectedEOF,
    #[error("Invalid descriptor format: {0}")]
    InvalidFormat(&'static str),
    #[error("Array has too many dimensions")]
    TooManyDimensions,
}

fn next(chars: &mut Chars) -> Result<Option<TypeDescriptor>, Error> {
    use TypeDescriptor::*;

    Ok(match chars.next().ok_or(Error::UnexpectedEOF)? {
        'Z' => Some(Boolean),
        'B' => Some(Byte),
        'C' => Some(Char),
        'S' => Some(Short),
        'I' => Some(Integer),
        'J' => Some(Long),
        'F' => Some(Float),
        'D' => Some(Double),
        'V' => Some(Void),
        'L' => {
            let (class_name, remainder) = chars.as_str().split_once(';').ok_or(
                Error::InvalidFormat("missing semicolon in class name descriptor"),
            )?;

            *chars = remainder.chars();
            Some(Object(class_name.to_string()))
        }
        '[' => {
            let extra = chars.as_str().chars().take_while(|&c| c == '[').count();

            let dimensions = (extra as u8)
                .checked_add(1)
                .ok_or(Error::TooManyDimensions)?;

            if extra > 0 {
                chars.nth(extra - 1);
            }

            let typ = next(chars)?.ok_or(Error::InvalidFormat("invalid arrat element type"))?;

            Some(Array(Box::new(typ), dimensions))
        }
        ')' => None,
        _ => return Err(Error::InvalidFormat("invalid type descriptor")),
    })
}

impl FromStr for MethodDescriptor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        (chars.next() == Some('('))
            .then_some(())
            .ok_or(Error::InvalidFormat(
                "method descriptor must start with a '('",
            ))?;

        let parameters =
            std::iter::from_fn(|| next(&mut chars).transpose()).collect::<Result<Vec<_>, _>>()?;

        let return_type = next(&mut chars)?.ok_or(Error::InvalidFormat("missing return type"))?;

        Ok(Self {
            parameters,
            return_type,
        })
    }
}

impl FromStr for TypeDescriptor {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        next(&mut s.chars())?.ok_or(Error::InvalidFormat("invalid descriptor"))
    }
}

impl From<TypeDescriptor> for Vec<i32> {
    fn from(value: TypeDescriptor) -> Self {
        match value {
            TypeDescriptor::Byte
            | TypeDescriptor::Char
            | TypeDescriptor::Integer
            | TypeDescriptor::Short
            | TypeDescriptor::Boolean => vec![0],
            TypeDescriptor::Float => vec![(0.0f32).to_bits() as i32],
            TypeDescriptor::Long => vec![0, 0],
            TypeDescriptor::Double => {
                let bits = (0.0f64).to_bits();
                vec![(bits >> 32) as i32, bits as i32]
            }
            TypeDescriptor::Array(_, _) | TypeDescriptor::Object(_) => vec![0],
            TypeDescriptor::Void => panic!("void doesn't have a value"),
        }
    }
}

impl std::fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for param in &self.parameters {
            write!(f, "{}", param)?;
        }
        write!(f, "){}", self.return_type)
    }
}

impl std::fmt::Display for TypeDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte => write!(f, "B"),
            Self::Char => write!(f, "C"),
            Self::Double => write!(f, "D"),
            Self::Float => write!(f, "F"),
            Self::Integer => write!(f, "I"),
            Self::Long => write!(f, "J"),
            Self::Short => write!(f, "S"),
            Self::Boolean => write!(f, "Z"),
            Self::Void => write!(f, "V"),
            Self::Array(base_type, dimensions) => {
                write!(f, "{}", "[".repeat(*dimensions as usize))?;
                write!(f, "{}", base_type)
            }
            Self::Object(class_name) => write!(f, "L{};", class_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TypeDescriptor::*;
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("B", Byte)]
    #[case("C", Char)]
    #[case("D", Double)]
    #[case("F", Float)]
    #[case("I", Integer)]
    #[case("J", Long)]
    #[case("S", Short)]
    #[case("Z", Boolean)]
    #[case("V", Void)]
    fn parses_primitive(#[case] input: &str, #[case] expected: TypeDescriptor) {
        assert_eq!(input.parse::<TypeDescriptor>().unwrap(), expected);
    }

    #[rstest]
    #[case("Ljava/lang/String;", Object("java/lang/String".to_string()))]
    #[case("Ljava/lang/Object;", Object("java/lang/Object".to_string()))]
    #[case("Lfoo/Bar;", Object("foo/Bar".to_string()))]
    fn parses_object(#[case] input: &str, #[case] expected: TypeDescriptor) {
        assert_eq!(input.parse::<TypeDescriptor>().unwrap(), expected);
    }

    #[rstest]
    #[case("[I", Array(Box::new(Integer), 1))]
    #[case("[[I", Array(Box::new(Integer), 2))]
    #[case("[[[I", Array(Box::new(Integer), 3))]
    #[case("[Ljava/lang/String;", Array(Box::new(Object("java/lang/String".to_string())), 1))]
    #[case("[[Ljava/lang/String;", Array(Box::new(Object("java/lang/String".to_string())), 2))]
    #[case("[Z", Array(Box::new(Boolean), 1))]
    fn parses_array(#[case] input: &str, #[case] expected: TypeDescriptor) {
        assert_eq!(input.parse::<TypeDescriptor>().unwrap(), expected);
    }

    #[rstest]
    #[case("B")]
    #[case("C")]
    #[case("D")]
    #[case("F")]
    #[case("I")]
    #[case("J")]
    #[case("S")]
    #[case("Z")]
    #[case("V")]
    #[case("Ljava/lang/String;")]
    #[case("[I")]
    #[case("[[[D")]
    #[case("[[Ljava/lang/Object;")]
    fn type_round_trips(#[case] descriptor: &str) {
        let parsed = descriptor.parse::<TypeDescriptor>().unwrap();
        assert_eq!(parsed.to_string(), descriptor);
    }

    #[rstest]
    #[case("", Error::UnexpectedEOF)]
    #[case("Q", Error::InvalidFormat("invalid type descriptor"))]
    #[case(
        "Ljava/lang/String",
        Error::InvalidFormat("missing semicolon in class name descriptor")
    )]
    fn type_parse_errors(#[case] input: &str, #[case] expected: Error) {
        let err = input.parse::<TypeDescriptor>().unwrap_err();
        assert_eq!(err.to_string(), expected.to_string());
    }

    #[rstest]
    #[case("()V", vec![], Void)]
    #[case("(I)I", vec![Integer], Integer)]
    #[case("(IJ)Z", vec![Integer, Long], Boolean)]
    #[case(
        "(Ljava/lang/String;[I)V",
        vec![Object("java/lang/String".to_string()), Array(Box::new(Integer), 1)],
        Void,
    )]
    #[case(
        "([[Ljava/lang/String;)Ljava/lang/Object;",
        vec![Array(Box::new(Object("java/lang/String".to_string())), 2)],
        Object("java/lang/Object".to_string()),
    )]
    fn parses_method(
        #[case] input: &str,
        #[case] parameters: Vec<TypeDescriptor>,
        #[case] return_type: TypeDescriptor,
    ) {
        let parsed = input.parse::<MethodDescriptor>().unwrap();
        assert_eq!(
            parsed,
            MethodDescriptor {
                parameters,
                return_type,
            }
        );
    }

    #[rstest]
    #[case("()V")]
    #[case("(I)I")]
    #[case("(IJ)Z")]
    #[case("(Ljava/lang/String;[I)V")]
    #[case("([[Ljava/lang/String;)Ljava/lang/Object;")]
    fn method_round_trips(#[case] descriptor: &str) {
        let parsed = descriptor.parse::<MethodDescriptor>().unwrap();
        assert_eq!(parsed.to_string(), descriptor);
    }

    #[rstest]
    #[case("V", Error::InvalidFormat("method descriptor must start with a '('"))]
    #[case("()", Error::UnexpectedEOF)]
    #[case("())", Error::InvalidFormat("missing return type"))]
    fn method_parse_errors(#[case] input: &str, #[case] expected: Error) {
        let err = input.parse::<MethodDescriptor>().unwrap_err();
        assert_eq!(err.to_string(), expected.to_string());
    }
}
