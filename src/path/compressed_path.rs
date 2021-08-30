use crate::Point;

use super::{Cost, Path};

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedPath {
    path: Arc<[u8]>,
    cost: Cost,
    pub(crate) start: Point,
    pub(crate) end: Point,
    length: usize,
    is_reversed: bool,
}

fn path_to_bytes(path: &[Point]) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::new();
    path.iter().for_each(|(x, y)| {
        ret.extend(x.to_le_bytes());
        ret.extend(y.to_le_bytes());
    });

    assert_eq!(path.len() * 16, ret.len());

    ret
}

impl CompressedPath {
    // pub fn new(path: Vec<Point>, cost: Cost) -> CompressedPath {
    //     let compressed_path = lz4_flex::compress(&path_to_bytes(&path));

    //     CompressedPath {
    //         path: compressed_path.into(),
    //         cost,
    //         is_reversed: false,
    //         start: path[0],
    //         end: path[path.len() - 1],
    //         length: path.len(),
    //     }
    // }

    pub fn cost(&self) -> Cost {
        self.cost
    }

    pub fn len(&self) -> usize {
        self.length
    }

    #[track_caller]
    pub fn get(&self, index: usize) -> Point {
        let idx = if self.is_reversed {
            self.len() - index - 1
        } else {
            index
        };
        self.decompress_path()[idx]
    }

    pub fn reversed(&self, start_cost: Cost, end_cost: Cost) -> CompressedPath {
        CompressedPath {
            path: self.path.clone(),
            cost: self.cost - start_cost + end_cost,
            is_reversed: !self.is_reversed,
            start: self.end,
            end: self.start,
            ..*self
        }
    }

    fn decompress_path(&self) -> Vec<Point> {
        let bytes =
            lz4_flex::decompress_size_prepended(&self.path).expect("Failed to decompress path.");

        use byteorder::{ByteOrder, LittleEndian};
        let chunk_size = std::mem::size_of::<usize>();

        let mut ret = Vec::new();

        for tuple in bytes.chunks_exact(chunk_size * 2) {
            let x: usize = LittleEndian::read_uint(&tuple[0..chunk_size], chunk_size) as usize;
            let y: usize =
                LittleEndian::read_uint(&tuple[chunk_size..chunk_size * 2], chunk_size) as usize;

            ret.push((x, y));
        }


        ret
    }
}

impl From<Path<Point>> for CompressedPath {
    fn from(path: Path<Point>) -> CompressedPath {
        let compressed_path = lz4_flex::compress_prepend_size(&path_to_bytes(&path.path));
        assert_eq!(path.len(), path.path.len());
        CompressedPath {
            path: compressed_path.into(),
            cost: path.cost(),
            start: path[0],
            end: path[path.len() - 1],
            is_reversed: path.is_reversed,
            length: path.len(),
        }
    }
}
