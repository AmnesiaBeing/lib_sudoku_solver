// // 坐标范围，0~8
// pub struct Coords {
//     // row id
//     r: usize,
//     // col id
//     c: usize,
// }

use std::{mem::MaybeUninit, ptr};

#[derive(Copy, Clone, PartialEq)]
pub enum CellStatus {
    // 固定数值
    FIXED,
    // 草稿，未填值
    DRAFT,
    // 用户的解答，已填值，此时drafts数组的内容将被忽略
    SOLVE,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CellValue {
    INVAILD = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
}

impl CellValue {
    fn from_value(v: u32) -> Result<CellValue, &'static str> {
        match v {
            0 => Ok(CellValue::INVAILD),
            1 => Ok(CellValue::V1),
            2 => Ok(CellValue::V2),
            3 => Ok(CellValue::V3),
            4 => Ok(CellValue::V4),
            5 => Ok(CellValue::V5),
            6 => Ok(CellValue::V6),
            7 => Ok(CellValue::V7),
            8 => Ok(CellValue::V8),
            9 => Ok(CellValue::V9),
            _ => Err("Invalid Input Number"),
        }
    }
}

pub struct Cell {
    status: CellStatus,
    drafts: [bool; 9],
    value: CellValue,
}

// pub struct Field {
//     pub raw_value: [Cell; 81],
// }

type Field = Box<[Cell; 81]>;

// pub fn get_unmut_cell_by_rc(&self, r: usize, c: usize) -> &Cell {
//     &self.raw_value[r * 9 + c]
// }

// pub fn get_cell_status_and_value_by_rc(&self, r: usize, c: usize) -> (CellStatus, CellValue) {
//     let p = self.raw_value[r * 9 + c];
//     (p.status, p.value)
// }

// pub fn set_cell_value_by_rc(&mut self, r: usize, c: usize, v: CellValue) -> Result<(), &str> {
//     let mut p = self.raw_value[r * 9 + c];
//     if p.status == CellStatus::FIXED {
//         Err("Invaild Operation")
//     } else {
//         p.value = v;
//         p.status = CellStatus::SOLVE;
//         Ok(())
//     }
// }

// pub fn set_cell_drafts_by_rc(&mut self, r: usize, c: usize, d: [bool; 9]) -> Result<(), &str> {
//     let mut p = self.raw_value[r * 9 + c];
//     if p.status == CellStatus::FIXED {
//         Err("Invalid Operation")
//     } else {
//         p.status = CellStatus::DRAFT;
//         p.drafts = d;
//         Ok(())
//     }
// }

// 解析输入长度为81的字符串序列
pub fn parse_by_string(input: String) -> Result<Field, &'static str> {
    let mut field: Field = unsafe {
        let mut cells = MaybeUninit::<[Cell; 81]>::uninit();
        let p = cells.as_mut_ptr() as *mut Cell;

        for i in 0..81 {
            ptr::write(
                p.offset(i),
                Cell {
                    status: CellStatus::DRAFT,
                    drafts: [true; 9],
                    value: CellValue::INVAILD,
                },
            )
        }
        Box::new(cells.assume_init())
    };

    let mut i: usize = 0;
    let mut j: usize = 0;

    for (_, item) in input.chars().enumerate() {
        let tmp = item.to_digit(10).expect("Invalid Character.");
        if tmp != 0 {
            field[i * 9 + j].value = CellValue::from_value(tmp)?;
            field[i * 9 + j].status = CellStatus::FIXED;
        }
        j = j + 1;
        if j >= 9 {
            i = i + 1;
            j = 0;
        }
    }
    if i * 9 + j != 81 {
        println!("{}", i * 9 + j);
        return Err("Invalid String Length.");
    }

    let _ = fill_drafts(&field);

    Ok(field)
}

// 初始化时，根据固定数字填充草稿
pub fn fill_drafts(field: &mut Field) -> Result<(), &'static str> {
    for r in 0..9 {
        for c in 0..9 {
            let p = &mut field[r * 9 + c];
            if p.status == CellStatus::FIXED || p.status == CellStatus::SOLVE {
                continue;
            }
            // else p.status == CellStatus::DRAFT
            // 按行检索
            {
                for r_iter in 0..9 {
                    if r_iter == r {
                        continue;
                    }
                    let ref p_iter = field[r_iter * 9 + c];
                    if p_iter.status == CellStatus::FIXED || p_iter.status == CellStatus::SOLVE {
                        p.drafts[p_iter.value as usize - 1] = false;
                    }
                }
            }
            // 按列检索
            {
                for c_iter in 0..9 {
                    if c_iter == c {
                        continue;
                    }
                    let ref p_iter = field[r * 9 + c_iter];
                    if p_iter.status == CellStatus::FIXED || p_iter.status == CellStatus::SOLVE {
                        p.drafts[p_iter.value as usize - 1] = false;
                    }
                }
            }
            // 按宫检索
            {}
        }
    }
    Ok(())
}

pub fn print(field: &Field) {
    println!("╔═══╤═══╤═══╦═══╤═══╤═══╦═══╤═══╤═══╗");
    for r in 0..9 {
        for m in 0..3 {
            let mut line: String = "║".to_string();
            for c in 0..9 {
                let p = field[r * 9 + c];
                if p.status == CellStatus::DRAFT {
                    for n in 0..3 {
                        let d = m * 3 + n;
                        if p.drafts[d] {
                            line += &((d + 1).to_string());
                        } else {
                            line += " ";
                        }
                    }
                } else if p.status == CellStatus::FIXED {
                    if m == 0 {
                        line += "\\ /";
                    } else if m == 1 {
                        line += " ";
                        line += &((p.value as u32).to_string());
                        line += " ";
                    } else {
                        line += "/ \\";
                    }
                }
                if c % 3 == 2 {
                    line += "║";
                } else {
                    line += "│";
                }
            }
            println!("{}", line);
        }
        if r == 8 {
            println!("╚═══╧═══╧═══╩═══╧═══╧═══╩═══╧═══╧═══╝")
        } else if r % 3 == 2 {
            println!("╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣")
        } else {
            println!("╟───┼───┼───╫───┼───┼───╫───┼───┼───╢")
        }
    }
}

// pub fn apply_1_approach(input: &Field) -> Result<Field, &'static str> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let field = parse_by_string(
            "005900060904532000007000900201000600040000010007000405009002000008514090060007500"
                .to_string(),
        )
        .unwrap();
        print(&field);
    }
}
