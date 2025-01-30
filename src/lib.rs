pub mod inferences;
pub mod types;
pub mod utils;

use types::Field;

use wasm_bindgen::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;

#[wasm_bindgen]
pub struct Sudoku {
    puzzle: [[u8; 9]; 9],
    solution: [[u8; 9]; 9],
}

#[wasm_bindgen]
impl Sudoku {
    #[wasm_bindgen(constructor)]
    pub fn new(difficulty: &str) -> Self {
        let (puzzle, solution) = generate_sudoku(difficulty);
        Sudoku { puzzle, solution }
    }

    #[wasm_bindgen(getter)]
    pub fn puzzle(&self) -> Vec<u8> {
        self.puzzle.iter().flatten().cloned().collect()
    }

    #[wasm_bindgen(getter)]
    pub fn solution(&self) -> Vec<u8> {
        self.solution.iter().flatten().cloned().collect()
    }
}

fn generate_sudoku(difficulty: &str) -> ([[u8; 9]; 9], [[u8; 9]; 9]) {
    let mut base = create_base_board();
    let (mut puzzle, mut solution) = dig_holes(&mut base, difficulty);
    shuffle_boards(&mut puzzle, &mut solution);
    (puzzle, solution)
}

fn create_base_board() -> [[u8; 9]; 9] {
    let mut board = [[0; 9]; 9];
    for i in 0..9 {
        for j in 0..9 {
            board[i][j] = ((i * 3 + i / 3 + j) % 9 + 1) as u8;
        }
    }
    board
}

fn dig_holes(base: &mut [[u8; 9]; 9], difficulty: &str) -> ([[u8; 9]; 9], [[u8; 9]; 9]) {
    let mut solution = *base;
    let mut puzzle = *base;
    
    phase1_dig(&mut puzzle);
    
    let max_digs = match difficulty {
        "easy" => 5,
        "medium" => 15,
        "hard" => 30,
        _ => 15,
    };
    phase2_dig(&mut puzzle, &solution, max_digs);
    
    (puzzle, solution)
}

fn phase1_dig(puzzle: &mut [[u8; 9]; 9]) {
    let mut positions: Vec<(usize, usize)> = (0..9).flat_map(|i| (0..9).map(move |j| (i, j))).collect();
    positions.shuffle(&mut rand::thread_rng());
    
    for (i, j) in positions {
        let original = puzzle[i][j];
        puzzle[i][j] = 0;
        if !check_uniqueness(&puzzle) {
            puzzle[i][j] = original;
        }
    }
}

fn phase2_dig(puzzle: &mut [[u8; 9]; 9], solution: &[[u8; 9]; 9], max_digs: usize) {
    let mut empty_cells: Vec<(usize, usize)> = puzzle.iter()
        .enumerate()
        .flat_map(|(i, row)| row.iter()
            .enumerate()
            .filter(|(_, &v)| v != 0)
            .map(move |(j, _)| (i, j)))
        .collect();
    empty_cells.shuffle(&mut rand::thread_rng());
    
    for (i, j) in empty_cells.into_iter().take(max_digs) {
        let original = puzzle[i][j];
        puzzle[i][j] = 0;
        if !check_uniqueness(&puzzle) {
            puzzle[i][j] = original;
        }
    }
}

fn shuffle_boards(puzzle: &mut [[u8; 9]; 9], solution: &mut [[u8; 9]; 9]) {
    let mut rng = rand::thread_rng();
    
    // 行交换
    for _ in 0..3 {
        let block = rng.gen_range(0..3);
        let mut rows: Vec<usize> = (block*3..(block+1)*3).collect();
        rows.shuffle(&mut rng);
        if let [r1, r2] = rows[..2] {
            puzzle.swap(r1, r2);
            solution.swap(r1, r2);
        }
    }
    
    // 列交换
    for _ in 0..3 {
        let block = rng.gen_range(0..3);
        let mut cols: Vec<usize> = (block*3..(block+1)*3).collect();
        cols.shuffle(&mut rng);
        if let [c1, c2] = cols[..2] {
            for row in puzzle.iter_mut() {
                row.swap(c1, c2);
            }
            for row in solution.iter_mut() {
                row.swap(c1, c2);
            }
        }
    }
    
    // 数字替换
    let mut numbers: Vec<u8> = (1..=9).collect();
    numbers.shuffle(&mut rng);
    let replace_map: Vec<u8> = (1..=9).map(|i| numbers[i as usize - 1]).collect();
    
    for i in 0..9 {
        for j in 0..9 {
            solution[i][j] = replace_map[solution[i][j] as usize - 1];
            if puzzle[i][j] != 0 {
                puzzle[i][j] = replace_map[puzzle[i][j] as usize - 1];
            }
        }
    }
}

fn check_uniqueness(puzzle: &[[u8; 9]; 9]) -> bool {
    let mut count = 0;
    let mut empty = vec![];
    
    for i in 0..9 {
        for j in 0..9 {
            if puzzle[i][j] == 0 {
                empty.push((i, j));
            }
        }
    }
    
    if empty.is_empty() {
        return true;
    }
    
    let (i, j) = empty[0];
    let candidates = get_candidates(puzzle, i, j);
    
    for num in candidates {
        let mut new_puzzle = *puzzle;
        new_puzzle[i][j] = num;
        if check_uniqueness(&new_puzzle) {
            count += 1;
            if count > 1 {
                return false;
            }
        }
    }
    
    count == 1
}

fn get_candidates(puzzle: &[[u8; 9]; 9], i: usize, j: usize) -> Vec<u8> {
    let mut used = HashSet::new();
    
    // 行检查
    for &num in &puzzle[i] {
        if num != 0 {
            used.insert(num);
        }
    }
    
    // 列检查
    for row in puzzle {
        let num = row[j];
        if num != 0 {
            used.insert(num);
        }
    }
    
    // 宫格检查
    let start_i = (i / 3) * 3;
    let start_j = (j / 3) * 3;
    for x in 0..3 {
        for y in 0..3 {
            let num = puzzle[start_i + x][start_j + y];
            if num != 0 {
                used.insert(num);
            }
        }
    }
    
    (1..=9).filter(|n| !used.contains(n)).collect()
}

#[cfg(test)]
mod tests {
    use crate::{inferences::InferenceSet, types::Field, utils::generate_combinations};

    fn sovle(field: &Field) {
        let mut field = field.clone();
        field.print();
        let ifs = InferenceSet::new();
        loop {
            let old_field = field.clone();
            let result = ifs.analyze(&old_field);
            match result {
                Some(result) => {
                    println!("{:?}", result);
                    InferenceSet::apply(&mut field, result);
                    field.print();
                    if let Some(conflict) = field.find_conflict() {
                        // field.print();
                        println!("conflict: {:?}", conflict);
                        // old_field.print();
                        break;
                    } else {
                        if field.check_if_finish() {
                            println!("推导完毕!");
                            // field.print();
                            break;
                        }
                    }
                }
                None => {
                    println!("无法推导!");
                    // field.print();
                    break;
                }
            }
        }
    }

    #[test]
    fn test1() {
        let field = Field::initial_by_string(
            &"070009800008002006906100000600000150030801020092000008000003701800600300001900060"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test2() {
        let field = Field::initial_by_string(
            &"615800790290600015040000260000080000730512046000090000080000030900008071071060582"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test3() {
        let field = Field::initial_by_string(
            &"010076000805000300000000000270000000000500100600000000003000002000900040000000076"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test4() {
        let field = Field::initial_by_string(
            &"800000000003600000070090200050007000000045700000100030001000068008500010090000400"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test5() {
        // 这个数独有不止一种答案，理论上应该推理不出来
        let field = Field::initial_by_string(
            &"000000100000500306000000500030600412060300958800000000000000000100000000000000000"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test6() {
        let field = Field::initial_by_string(
            &"060000000100000054000000700003000001008010070051000000080900000007100000010000000"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test7() {
        let field = Field::initial_by_string(
            &"586000020020465873437020516300710068008000100010082000073090045000000390090253080"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test8() {
        let field = Field::initial_by_string(
            &"900400613320190700000000009000017008000000000700360000800000000009045086253001004"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test9() {
        let field = Field::initial_by_string(
            &"400090708007810400080060050800130007000070000170028005068051024513249876042080501"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test10() {
        let field = Field::initial_by_string(
            &"000000000000010000012304560000000000035000780081020350000000000057000630063807210"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test11() {
        let field = Field::initial_by_string(
            &"807530429935427681240900375483652917672193854009874236020340708308710542704200103"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn test12() {
        let field = Field::initial_by_string(
            &"000000000000010000012304560000000000035000780081020350000000000057000630063807210"
                .to_string(),
        )
        .unwrap();
        sovle(&field);
    }

    #[test]
    fn generate_combinations_test() {
        let mut all_combinations = Vec::new();
        for size in 2..=4 {
            let mut paths = Vec::new();
            generate_combinations(9, size, 0, &mut paths, &mut all_combinations);
        }
        println!("{:?}", all_combinations);
    }

    fn is_n_fish_pair(v1: &Vec<(usize, usize)>, v2: &Vec<(usize, usize)>) -> bool {
        if v2.is_empty() {
            false
        } else {
            v2.iter()
                .all(|&(_, v2_c)| v1.iter().any(|&(_, v1_c)| v2_c == v1_c))
        }
    }

    #[test]
    fn is_n_fish_pair_test() {
        let v1 = vec![(0, 1), (0, 5)];
        let v2 = vec![(7, 1), (7, 5)];
        assert!(is_n_fish_pair(&v1, &v2) == true);
    }

    #[test]
    fn test_initial_by_random() {
        Field::initial_by_random();
    }
}
