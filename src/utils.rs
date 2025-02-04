// 这里放置一些常用的工具类

use crate::{
    inferences::TheCoordsAndTheValue,
    types::{Cell, Coords, GNCoords, RCCoords, Sudoku},
};

// 定义子函数，将一个集合拆分成X和剩余部分的两个集合，且 2<=X<=4
// 这里生成长度为2/3/4的所有组合的数组索引
pub fn generate_combinations(
    full_length: usize,
    combo_size: usize,
    current: usize,
    path: &mut Vec<usize>,
    all_combinations: &mut Vec<(Vec<usize>, Vec<usize>)>,
) {
    // 剪枝：当数对和需要组合的长度相等时，直接返回，没有必要进行判断了
    if path.len() == full_length {
        return;
    }
    if path.len() == combo_size {
        let remaining: Vec<usize> = (0..full_length).filter(|i| !path.contains(i)).collect();
        all_combinations.push((path.clone(), remaining));
        return;
    }
    for i in current..full_length {
        path.push(i);
        generate_combinations(full_length, combo_size, i + 1, path, all_combinations);
        path.pop();
    }
}

// 当某个格子设置某个值的时候，将同行列宫的该值的草稿值移除，输入值在vec_set_value.cells内，且value唯一
pub fn make_simple_conclusion_when_set_value<'a>(
    field: &'a Sudoku,
    coords: &'a Coords,
    value: u8,
) -> Option<Vec<TheCoordsAndTheValue>> {
    let ret: Vec<TheCoordsAndTheValue> = field
        .collect_all_drafts_coords_by_coords_and_value(*coords, value)
        .iter()
        .map(|&p| create_simple_cell_and_value(p, value))
        .collect();

    if !ret.is_empty() {
        Some(ret)
    } else {
        None
    }
}

pub fn create_simple_cell_and_value<'a>(coords: Coords, v: u8) -> TheCoordsAndTheValue {
    TheCoordsAndTheValue {
        the_coords: coords,
        the_value: vec![v],
    }
}

#[derive(Debug)]
pub enum IterDirection {
    Row,
    Column,
    Grid,
}

pub fn get_rc_coord_with_direction(
    one_index: usize,
    other_index: usize,
    direction: &IterDirection,
) -> RCCoords {
    match direction {
        IterDirection::Row => RCCoords {
            r: one_index,
            c: other_index,
        },
        IterDirection::Column => RCCoords {
            r: other_index,
            c: one_index,
        },
        // 正常不应该到这里来
        IterDirection::Grid => todo!(),
    }
}

pub fn get_rc_index_with_direction(rc: RCCoords, direction: &IterDirection) -> usize {
    match direction {
        IterDirection::Row => rc.r,
        IterDirection::Column => rc.c,
        // 正常不应该到这里来
        IterDirection::Grid => todo!(),
    }
}

pub fn get_one_index_with_direction(coords: Coords, direction: &IterDirection) -> usize {
    let Coords { r, c, g, n: _ } = coords;
    match direction {
        IterDirection::Row => r,
        IterDirection::Column => c,
        IterDirection::Grid => g,
    }
}

pub fn get_other_index_with_direction(coords: Coords, direction: &IterDirection) -> usize {
    let Coords { r, c, g: _, n } = coords;
    match direction {
        IterDirection::Row => c,
        IterDirection::Column => r,
        IterDirection::Grid => n,
    }
}

pub fn get_coords_with_direction(
    one_index: usize,
    other_index: usize,
    direction: &IterDirection,
) -> Coords {
    match direction {
        IterDirection::Row => RCCoords {
            r: one_index,
            c: other_index,
        }
        .into(),
        IterDirection::Column => RCCoords {
            r: other_index,
            c: one_index,
        }
        .into(),
        IterDirection::Grid => GNCoords {
            g: one_index,
            n: other_index,
        }
        .into(),
    }
}
