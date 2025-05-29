use std::io::{Read, Seek};

use anyhow::ensure;
use binrw::{binread, BinRead};

#[derive(BinRead, Debug)]
#[br(magic = b".MESSIAH")]
pub struct MessiahHeader {
    // 8 for models
    pub file_type: MessiahFileType,
    pub file_size: u32,
}

#[derive(BinRead, Debug, PartialEq, Eq)]
#[br(repr(u32))]
pub enum MessiahFileType {
    Unk0 = 0,     // Related to occluders, Example: c0bb2bee-de22-460b-91e8-137d7f39f433
    Unk1 = 1,     // Example: cea70cc4-5c44-5f24-9b93-996a95f12fe5
    Unk2 = 2,     // Example: 692937cf-de53-32a7-ab7e-0a5cc802034b
    Unk3 = 3,     // Example: 67af9cef-db4f-1dfd-2cc0-e42e90d475cc
    Material = 4, // Example: 7e87d7d2-defb-5def-a499-abdf244e83d2
    Unk6 = 6,     // Example: dd281138-eeb3-4a71-962d-dfc18907ae7a
    Model = 8,
    Unk15 = 15, // Large complex, Example: 11e623a8-b03d-122d-a23c-c17713bf175d
}

#[derive(BinRead, Debug)]
pub struct ModelHeader {
    pub unk10: u16,
    pub mesh_count: u16,
    pub vertex_count: u32,
    pub index_count: u32,
    pub buffer_layouts: [BufferLayoutString; 4],
    pub numbers: [f32; 10],

    #[br(count = mesh_count as usize)]
    pub meshes: Vec<MeshOffsets>,

    index_offset: binrw::PosValue<()>,
    // #[br(count = index_count as usize)]
    // pub indices: Vec<u16>,
}

impl ModelHeader {
    pub fn index_format(&self) -> IndexFormat {
        if self.vertex_count <= 0xffff {
            IndexFormat::U16
        } else {
            IndexFormat::U32
        }
    }

    /// Reads the index buffer, converting u16s to u32s
    pub fn read_indices_u32<R: Read + Seek>(&self, mut reader: R) -> anyhow::Result<Vec<u32>> {
        match self.index_format() {
            IndexFormat::U16 => {
                let mut indices = vec![0u16; self.index_count as usize];
                reader.read_exact(bytemuck::cast_slice_mut(&mut indices))?;
                Ok(indices.into_iter().map(|i| i as u32).collect())
            }
            IndexFormat::U32 => {
                let mut indices = vec![0u32; self.index_count as usize];
                reader.read_exact(bytemuck::cast_slice_mut(&mut indices))?;
                Ok(indices)
            }
        }
    }

    pub fn index_buffer_offset(&self) -> u64 {
        self.index_offset.pos
    }
    pub fn vertex_buffer_offset(&self) -> u64 {
        self.index_buffer_offset()
            + (self.index_count as usize * self.index_format().stride()) as u64
    }
}

pub enum IndexFormat {
    U16,
    U32,
}

impl IndexFormat {
    pub fn stride(&self) -> usize {
        match self {
            IndexFormat::U16 => 2,
            IndexFormat::U32 => 4,
        }
    }
}

#[derive(BinRead, Debug)]
pub struct MeshOffsets {
    pub offset0: u32,
    pub offset1: u32,
    pub offset2: u32,
    pub offset3: u32,
}

#[binread]
#[derive(Debug)]
pub struct BufferLayoutString {
    #[br(temp)]
    length: u16,
    #[br(map = |v: Vec<u8>| String::from_utf8_lossy(&v).to_string(), count = length as usize)]
    pub string: String,
}

impl BufferLayoutString {
    pub fn to_buffer_layout(&self) -> anyhow::Result<Option<BufferLayout>> {
        if &self.string == "None" {
            return Ok(None);
        }

        let mut elements = Vec::new();
        for part in self.string.split("_") {
            let components = part.chars().collect::<Vec<_>>();
            ensure!(components.len() == 3);
            elements.push(BufferLayoutElement {
                kind: LayoutType::try_from(components[0])?,
                elements: components[1].to_digit(10).unwrap() as u8,
                data_type: LayoutDataType::try_from(components[2])?,
            });
        }

        Ok(Some(BufferLayout { elements }))
    }
}

#[derive(Debug)]
pub struct BufferLayout {
    pub elements: Vec<BufferLayoutElement>,
}

impl BufferLayout {
    pub fn stride(&self) -> usize {
        self.elements.iter().map(|e| e.size()).sum()
    }
}

#[derive(Debug)]
pub struct BufferLayoutElement {
    pub kind: LayoutType,
    pub elements: u8,
    pub data_type: LayoutDataType,
}

impl BufferLayoutElement {
    pub fn size(&self) -> usize {
        self.data_type.size() * self.elements as usize
    }
}

#[derive(Debug)]
pub enum LayoutType {
    Position,
    Normal,
    Color,
    Texcoord,
    Tangent,
    Bitangent,
    BoneIndex,
    BoneWeight,
}

impl TryFrom<char> for LayoutType {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, anyhow::Error> {
        match value {
            'P' => Ok(Self::Position),
            'N' => Ok(Self::Normal),
            'C' => Ok(Self::Color),
            'T' => Ok(Self::Texcoord),
            'A' => Ok(Self::Tangent),
            'B' => Ok(Self::Bitangent),
            'I' => Ok(Self::BoneIndex),
            'W' => Ok(Self::BoneWeight),
            _ => Err(anyhow::anyhow!("Unknown layout type character {value}")),
        }
    }
}

#[derive(Debug)]
pub enum LayoutDataType {
    Float,
    UShort,
    UByte,
}

impl LayoutDataType {
    pub fn size(&self) -> usize {
        match self {
            Self::Float => 4,
            Self::UShort => 2,
            Self::UByte => 1,
        }
    }
}

impl TryFrom<char> for LayoutDataType {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, anyhow::Error> {
        match value {
            'F' => Ok(Self::Float),
            'H' => Ok(Self::UShort),
            'B' => Ok(Self::UByte),
            _ => Err(anyhow::anyhow!(
                "Unknown layout data type character {value}"
            )),
        }
    }
}
