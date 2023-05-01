use std::io::Write;
use std::string;

use diesel::backend::{Backend, self};
use diesel::backend::SqlDialect;
use diesel::pg::{self, Pg};
use diesel::sql_types::{
    Serial,
    Char,
    VarChar,
};
use strum_macros::AsRefStr;


#[derive(Debug)]
pub enum Field {
    Char {
        max_length: usize,
    },
    VarChar,
    Serial,
    BigInt,
    BigSerial,
    Text,
    Boolean,
    Bit {
        length: usize
    },
}


impl IntoSql<pg::Pg> for Field {
    fn into_sql<W: Write>(&self, writer: &mut W) -> Result<usize, Box<dyn std::error::Error>> {
        let data_type = match self {
            Self::Char { max_length } => format!("CHAR({})", *max_length),
            Self::VarChar => "VARCHAR".into(),
            Self::Text => "TEXT".into(),
            Self::Serial => "SERIAL".into(),
            Self::BigInt => "BIGINT".into(),
            Self::BigSerial => "BIGSERIAL".into(),
            Self::Boolean => "BOOLEAN".into(),
            Self::Bit { length } => format!("BIT({})", *length),
        };

        writer
        .write(data_type.as_bytes())
        .map_err(|err| err.into())
    }
}

#[derive(Debug)]
pub struct TableField {
    options: CommonFieldOptions,
    kind: Field
}

impl IntoSql<Pg> for TableField {
    fn into_sql<W: Write>(&self, writer: &mut W) -> Result<usize, Box<dyn std::error::Error>> {
        let mut total_bytes = 0;
        total_bytes += writer.write(self.options.name.as_bytes())?;
        total_bytes += writer.write(" ".as_bytes())?;

        total_bytes += self.kind.into_sql(writer)?;
        total_bytes += writer.write(" ".as_bytes())?;
        
        if let Some(null_constraint) = self.options.null {
            let value = if null_constraint {
                "NULL"
            } else {
                "NOT NULL"
            };
            total_bytes += writer.write(value.as_bytes())?;
            total_bytes += writer.write(" ".as_bytes())?;
        }

        if self.options.primary_key {
            total_bytes += writer.write("PRIMARY KEY".as_bytes())?;
            total_bytes += writer.write(" ".as_bytes())?;
        }
        if self.options.unique {
            total_bytes += writer.write("UNIQUE".as_bytes())?;
            total_bytes += writer.write(" ".as_bytes())?;
        }

        Ok(total_bytes)
    }
}


#[derive(Debug)]
pub struct CommonFieldOptions {
    name: String,
    primary_key: bool,
    unique: bool,
    null: Option<bool>,
}

#[derive(Debug, AsRefStr)]
pub enum TableKind {
    Global,
    Local
}

impl<T> IntoSql<pg::Pg> for T 
where
    T: AsRef<str>
{
    fn into_sql<W: Write>(&self, writer: &mut W) -> Result<usize, Box<dyn std::error::Error>> {
        writer.write(self.as_ref().to_uppercase().as_bytes()).map_err(|err| err.into())
    }
}


#[derive(Debug)]
pub struct CommonTableOptions {
    name: String,
    if_not_exists: bool,
    kind: Option<TableKind>,
}


pub trait IntoSql<B: Backend> {
    fn into_sql<W: Write>(&self, writer: &mut W) -> Result<usize, Box<dyn std::error::Error>>;

    fn into_sql_str(&self) -> Result<(String, usize), Box<dyn std::error::Error>> {
        let mut writer = Vec::new();
        let bytes_written = self.into_sql(&mut writer)?;
        Ok((String::from_utf8(writer)?, bytes_written))
    }
}

pub struct TableDefn {
    fields: Vec<TableField>,
    options: CommonTableOptions
}

impl IntoSql<pg::Pg> for TableDefn {
    fn into_sql<W: Write>(&self, writer: &mut W) -> Result<usize, Box<dyn std::error::Error>> {
        
        let mut total_bytes = 0;
        total_bytes += writer.write("CREATE ".as_bytes())?;

        if let Some(kind) = &self.options.kind {
            total_bytes += kind.into_sql(writer)?;
            total_bytes += writer.write(b" ")?;
        };

        total_bytes += writer.write(self.options.name.as_bytes())?;
        if self.options.if_not_exists {
            total_bytes += writer.write(b" IF NOT EXISTS")?;
        }

        total_bytes += writer.write(b" (\n\t")?;


        let num_fields = self.fields.len();

        self
        .fields
        .iter()
        .enumerate()
        .for_each(|(index, field)| {
            total_bytes += field.into_sql(writer).unwrap();
            if index != num_fields - 1 {
                total_bytes += writer.write(b",\n\t").unwrap();
            }
        });

        total_bytes += writer.write(b"\n)")?;
        Ok(total_bytes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table() {

        let posts = TableDefn {
            options: CommonTableOptions { name: "posts".to_string(), if_not_exists: true, kind: Some(TableKind::Global) },
            fields: vec![
                TableField {
                    options: CommonFieldOptions {
                        name: "id".to_string(),
                        primary_key: true,
                        unique: false,
                        null: None
                    },
                    kind: Field::Serial
                },
                TableField {
                    options: CommonFieldOptions {
                        name: "title".to_string(),
                        primary_key: false,
                        unique: false,
                        null: Some(false)
                    },
                    kind: Field::Char { max_length: 10 }
                },
                TableField {
                    options: CommonFieldOptions {
                        name: "body".to_string(),
                        primary_key: false,
                        unique: false,
                        null: Some(false)
                    },
                    kind: Field::Text
                },
                TableField {
                    options: CommonFieldOptions {
                        name: "published".to_string(),
                        primary_key: false,
                        unique: false,
                        null: Some(false),
                    },
                    kind: Field::Boolean
                },
            ],
        };
        let (observed, _) = posts.into_sql_str().unwrap();
        let expected = "CREATE GLOBAL posts IF NOT EXISTS (\n\tid SERIAL PRIMARY KEY ,\n\ttitle CHAR(10) NOT NULL ,\n\tbody TEXT NOT NULL ,\n\tpublished BOOLEAN NOT NULL \n)";
        assert_eq!(observed, expected);
    }
}