use std::convert::TryFrom;

use super::Direction;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OneDimensionalDirection {
    #[default]
    East,
    West,
}

impl OneDimensionalDirection {
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::default()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OneDimensionConvertError {
    North,
    South,
    Here,
}

impl TryFrom<Direction> for OneDimensionalDirection {
    type Error = OneDimensionConvertError;

    fn try_from(value: Direction) -> Result<Self, Self::Error> {
        match value {
            Direction::NorthEast | Direction::East | Direction::SouthEast => Ok(Self::East),
            Direction::SouthWest | Direction::West | Direction::NorthWest => Ok(Self::West),
            Direction::North => Err(OneDimensionConvertError::North),
            Direction::South => Err(OneDimensionConvertError::South),
            Direction::Here => Err(OneDimensionConvertError::Here),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::{Direction, OneDimensionConvertError, OneDimensionalDirection};

    #[test]
    fn south_east_to_two_dim() {
        let dir = OneDimensionalDirection::try_from(Direction::SouthEast);
        assert!(matches!(dir, Ok(OneDimensionalDirection::East)));
    }

    #[test]
    fn west_to_two_dim() {
        let dir = OneDimensionalDirection::try_from(Direction::West);
        assert!(matches!(dir, Ok(OneDimensionalDirection::West)));
    }

    #[test]
    fn north_to_two_dim() {
        let dir = OneDimensionalDirection::try_from(Direction::North);
        assert!(matches!(dir, Err(OneDimensionConvertError::North)));
    }
}
