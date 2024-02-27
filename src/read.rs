use std::io::Read;

fn find_newline(data: &[u8]) -> Option<usize> {
    data.iter()
        .enumerate()
        .find(|(_, b)| **b == b'\n')
        .map(|(i, _)| i)
}

#[derive(Clone, Copy, Debug)]
enum HeaderReaderState {
    Read0,
    Read1,
    ReadLine,
    IgnoreLine,
    End,
}

pub struct HeaderReader<R> {
    inner: R,
    buf: Vec<u8>,
    read_pos: usize,
    filled: usize,
    state: HeaderReaderState,
}

impl<R> HeaderReader<R> {
    const BUF_SIZE: usize = 1024;

    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buf: vec![0; Self::BUF_SIZE],
            read_pos: 0,
            filled: 0,
            state: HeaderReaderState::Read0,
        }
    }
}

impl<R: Read> HeaderReader<R> {
    pub fn into_body_reader(mut self) -> Result<BodyReader<R>, std::io::Error> {
        self.read_to_end()?;

        let already_read = (self.read_pos < self.filled).then(|| {
            AlreadyReadPart {
                buf: self.buf,
                start: self.read_pos,
                end: self.filled,
            }
        });

        Ok(BodyReader::new(self.inner, already_read))
    }

    fn read_internal<'a>(&'a mut self, dest: Option<&mut [u8]>) -> Result<usize, std::io::Error> {
        loop {
            if self.read_pos == self.filled {
                let n = self.inner.read(&mut self.buf)?;
                self.filled = n;
                self.read_pos = 0;

                if self.filled == 0 {
                    self.state = HeaderReaderState::End;
                    return Ok(0);
                }
            }

            assert!(self.filled - self.read_pos >= 1);
            let data = &self.buf[self.read_pos..self.filled];

            match self.state {
                HeaderReaderState::Read0 => {
                    if data[0] == b'#' {
                        self.read_pos += 1;
                        self.state = HeaderReaderState::Read1;
                    }
                    else {
                        self.state = HeaderReaderState::End;
                    }
                }
                HeaderReaderState::Read1 => {
                    if data[0] == b'#' {
                        self.read_pos += 1;
                        self.state = HeaderReaderState::IgnoreLine;
                    }
                    else if data[0] == b' ' {
                        self.read_pos += 1;
                        self.state = HeaderReaderState::ReadLine;
                    }
                    else {
                        return Err(std::io::ErrorKind::InvalidData.into());
                    }
                }
                HeaderReaderState::ReadLine => {
                    let mut n = data.len();

                    if let Some(dest) = &dest {
                        if dest.len() < n {
                            n = dest.len();
                        }
                    }

                    if let Some(new_line_pos) = find_newline(&data[..n]) {
                        n = new_line_pos + 1;
                        self.state = HeaderReaderState::Read0;
                    }

                    self.read_pos += n;

                    if let Some(dest) = dest {
                        dest[..n].copy_from_slice(&data[..n]);
                    }

                    return Ok(n);
                }
                HeaderReaderState::IgnoreLine => {
                    let mut n = data.len();
                    if let Some(new_line_pos) = find_newline(&data[..n]) {
                        n = new_line_pos + 1;
                        self.state = HeaderReaderState::Read0;
                    }
                    self.read_pos += n;
                }
                HeaderReaderState::End => {
                    return Ok(0);
                }
            }
        }
    }

    fn read_to_end(&mut self) -> Result<(), std::io::Error> {
        while self.read_internal(None)? > 0 {}
        Ok(())
    }
}

impl<R: Read> Read for HeaderReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_internal(Some(buf))
    }
}

#[derive(Clone, Copy, Debug)]
enum BodyReaderState {
    Read0,
    ReadLine,
    IgnoreLine,
    End,
}

pub struct BodyReader<R> {
    inner: RawBodyReader<R>,
    buf: Vec<u8>,
    read_pos: usize,
    filled: usize,
    state: BodyReaderState,
}

impl<R> BodyReader<R> {
    const BUF_SIZE: usize = 1024;

    fn new(inner: R, already_read: Option<AlreadyReadPart>) -> Self {
        Self {
            inner: RawBodyReader::new(inner, already_read),
            buf: vec![0; Self::BUF_SIZE],
            read_pos: 0,
            filled: 0,
            state: BodyReaderState::Read0,
        }
    }
}

impl<R: Read> Read for BodyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.read_pos == self.filled {
                let n = self.inner.read(&mut self.buf)?;
                self.filled = n;
                self.read_pos = 0;

                if self.filled == 0 {
                    self.state = BodyReaderState::End;
                    return Ok(0);
                }
            }

            assert!(self.filled - self.read_pos >= 1);
            let data = &self.buf[self.read_pos..self.filled];

            match self.state {
                BodyReaderState::Read0 => {
                    if data[0] == b'#' {
                        self.read_pos += 1;
                        self.state = BodyReaderState::IgnoreLine;
                    }
                    else {
                        self.state = BodyReaderState::ReadLine;
                    }
                }
                BodyReaderState::ReadLine => {
                    // todo: we need to ignore lines that are only whitespace, but for that we need
                    // to read the whole line :/
                    let mut n = data.len();

                    if buf.len() < n {
                        n = buf.len();
                    }

                    if let Some(new_line_pos) = find_newline(&data[..n]) {
                        n = new_line_pos + 1;
                        self.state = BodyReaderState::Read0;
                    }

                    self.read_pos += n;
                    buf[..n].copy_from_slice(&data[..n]);

                    return Ok(n);
                }
                BodyReaderState::IgnoreLine => {
                    let mut n = data.len();

                    if let Some(new_line_pos) = find_newline(&data[..n]) {
                        n = new_line_pos + 1;
                        self.state = BodyReaderState::Read0;
                    }

                    self.read_pos += n;
                }
                BodyReaderState::End => {
                    return Ok(0);
                }
            }
        }
    }
}

struct AlreadyReadPart {
    buf: Vec<u8>,
    start: usize,
    end: usize,
}

pub struct RawBodyReader<R> {
    inner: R,
    already_read: Option<AlreadyReadPart>,
}

impl<R> RawBodyReader<R> {
    fn new(inner: R, already_read: Option<AlreadyReadPart>) -> Self {
        Self {
            inner,
            already_read,
        }
    }
}

impl<R: Read> Read for RawBodyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(already_read) = &mut self.already_read {
            let n = std::cmp::min(already_read.end - already_read.start, buf.len());
            buf.copy_from_slice(&already_read.buf[already_read.start..][..n]);
            already_read.start += n;
            if already_read.start >= already_read.end {
                self.already_read = None;
            }
            Ok(n)
        }
        else {
            self.inner.read(buf)
        }
    }
}
