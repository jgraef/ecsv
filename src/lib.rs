mod error;
pub mod header;
mod read;

use std::io::{
    BufRead,
    BufReader,
    Read,
};

pub use error::Error;
use header::Delimiter;
use read::BodyReader;

use crate::{
    header::Header,
    read::HeaderReader,
};

pub struct Ecsv<R> {
    pub reader: csv::Reader<BodyReader<R>>,
    pub header: Header,
}

pub fn read<R: Read>(reader: R) -> Result<Ecsv<R>, Error> {
    let mut header_reader = BufReader::new(HeaderReader::new(reader));

    let mut buf = String::new();
    header_reader.read_line(&mut buf)?;
    if buf.trim() != "%ECSV 1.0" {
        return Err(Error::InvalidSignature);
    }

    buf.clear();
    header_reader.read_line(&mut buf)?;
    if buf.trim() != "---" {
        return Err(Error::InvalidSignature);
    }

    let header: Header = serde_yaml::from_reader(&mut header_reader)?;
    let delimiter = match header.delimiter {
        Delimiter::Space => b' ',
        Delimiter::Comma => b',',
    };

    let body_reader = header_reader.into_inner().into_body_reader()?;
    let reader = csv::ReaderBuilder::default()
        .delimiter(delimiter)
        .from_reader(body_reader);

    Ok(Ecsv { reader, header })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn it_parses_a_simple_ecsv_file() {
        let ecsv = r#"# %ECSV 1.0
# ---
# datatype:
# - {name: a, unit: m / s, datatype: float64, format: '%5.2f', description: Column A}
# - name: b
#   datatype: int64
#   meta:
#     column_meta: {a: 1, b: 2}
# meta: !!omap
# - keywords: !!omap
#   - {z_key1: val1}
#   - {a_key2: val2}
# - comments: [Comment 1, Comment 2, Comment 3]
# schema: astropy-2.0
a b
1.0 2
4.0 3
"#;
        let file = Cursor::new(ecsv.as_bytes());
        let ecsv = read(file).unwrap();
        println!("{:#?}", ecsv.header);
    }
}
