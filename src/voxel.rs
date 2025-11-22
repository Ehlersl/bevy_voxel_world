use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

pub const VOXEL_SIZE: f32 = 1.;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum WorldVoxel<I = u8> {
    #[default]
    Unset,
    Air,
    Solid(I),
}

impl<I: PartialEq> WorldVoxel<I> {
    pub fn is_unset(&self) -> bool {
        *self == WorldVoxel::Unset
    }

    pub fn is_air(&self) -> bool {
        *self == WorldVoxel::Air
    }

    pub fn is_solid(&self) -> bool {
        matches!(self, WorldVoxel::Solid(_))
    }
}

impl<I: PartialEq> Voxel for WorldVoxel<I> {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == WorldVoxel::Air || *self == WorldVoxel::Unset {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl<I: PartialEq + Eq + Default + Copy> MergeVoxel for WorldVoxel<I> {
    type MergeValue = I;

    fn merge_value(&self) -> Self::MergeValue {
        match self {
            WorldVoxel::Solid(v) => *v,
            _ => I::default(),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum VoxelFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl VoxelFace {
    pub fn is_top(self) -> bool {
        matches!(self, VoxelFace::Top)
    }

    pub fn is_bottom(self) -> bool {
        matches!(self, VoxelFace::Bottom)
    }

    pub fn is_side(self) -> bool {
        !matches!(self, VoxelFace::Top | VoxelFace::Bottom)
    }

    pub fn atlas_column(self) -> usize {
        match self {
            VoxelFace::Top => 0,
            VoxelFace::Bottom => 1,
            VoxelFace::North | VoxelFace::South | VoxelFace::East | VoxelFace::West => 2,
        }
    }
}
