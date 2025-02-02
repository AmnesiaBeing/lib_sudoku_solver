use crate::{
    types::{Cell, CellStatus, CellValue, Coords, Drafts, Sudoku, GNCoords, RCCoords},
    utils::{
        create_simple_cell_and_value, get_coords_with_direction, get_rc_coord_with_direction,
        make_simple_conclusion_when_set_value, IterDirection,
    },
};

/// 某某策略的结论通常可以归纳为：因为【某个地方的某个值】，导致【某个地方的某个值】，需要做一些什么
/// 这里定义的是【某个地方的某个值】
#[derive(Clone)]
pub struct TheCoordsAndTheValue {
    pub the_coords: Coords,
    pub the_value: Vec<CellValue>,
}

// impl std::fmt::Debug for TheCoordsAndTheValue {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{:?}V{:?}", self.the_coords.rc, self.the_value)
//     }
// }

pub struct InferenceResult<'a> {
    inference: &'a dyn Inference,
    condition: Vec<TheCoordsAndTheValue>,
    conclusion_set_value: Option<Vec<TheCoordsAndTheValue>>,
    conclusion_remove_drafts: Option<Vec<TheCoordsAndTheValue>>,
}

trait Inference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>>;
    fn write_result(&self, inference_result: &InferenceResult) -> String;
}

pub struct InferenceSet {
    inferences: Vec<Box<dyn Inference>>,
}

impl InferenceSet {
    pub fn new() -> Self {
        InferenceSet {
            inferences: vec![
                Box::new(OnlyOneLeftInference),
                Box::new(OnlyOneRightInRowInference),
                Box::new(OnlyOneRightInColInference),
                Box::new(OnlyOneRightInGridInference),
                Box::new(RowUniqueDraftByGridExclusionInference),
                Box::new(ColUniqueDraftByGridExclusionInference),
                Box::new(GridUniqueDraftByRowExclusionInference),
                Box::new(GridUniqueDraftByColExclusionInference),
                Box::new(RowExplicitNakedPairExclusionInference),
                Box::new(ColExplicitNakedPairExclusionInference),
                Box::new(GridExplicitNakedPairExclusionInference),
                Box::new(RowExplicitHiddenPairExclusionInference),
                Box::new(ColExplicitHiddenPairExclusionInference),
                Box::new(GridExplicitHiddenPairExclusionInference),
                Box::new(NStepFishInference),
                Box::new(ExploitInference),
            ],
        }
    }

    pub fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        self.inferences.iter().find_map(|inf| inf.analyze(field))
    }

    pub fn apply(field: &mut Sudoku, result: InferenceResult) {
        if result.conclusion_set_value.is_some() {
            result.conclusion_set_value.unwrap().iter().for_each(|cv| {
                let p = field.get_cell_mut_by_coords(cv.the_coords);
                p.value = cv.the_value[0];
                p.status = CellStatus::SOLVE;
            })
        };
        if result.conclusion_remove_drafts.is_some() {
            result
                .conclusion_remove_drafts
                .unwrap()
                .iter()
                .for_each(|cv| {
                    let p = field.get_cell_mut_by_coords(cv.the_coords);
                    cv.the_value.iter().for_each(|&v| p.drafts.remove_draft(v));
                })
        }
    }
}

impl<'a> std::fmt::Debug for InferenceResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inference.write_result(&self))
    }
}

/// 唯余法，遍历所有草稿单元格，如果存在唯一草稿，则说明这个草稿填写该数字
#[derive(Clone)]
struct OnlyOneLeftInference;
impl Inference for OnlyOneLeftInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.collect_all_drafts_cells().iter().find_map(|&p| {
            p.drafts.try_get_the_only_one().map(|cv| {
                let condition = TheCoordsAndTheValue {
                    the_coords: p.coords,
                    the_value: vec![cv],
                };
                InferenceResult {
                    inference: self,
                    condition: vec![condition.clone()],
                    conclusion_set_value: Some(vec![condition.clone()]),
                    conclusion_remove_drafts: make_simple_conclusion_when_set_value(
                        field,
                        &condition.the_coords,
                        condition.the_value[0],
                    ),
                }
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在格内唯一，因此 {:?} 只能填写 {:?} ",
            Into::<RCCoords>::into(condition.the_coords),
            condition.the_value[0],
            Into::<RCCoords>::into(condition.the_coords),
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                    .collect::<Vec<String>>()
                    .join(" "),
                condition.the_value[0]
            ));
        }

        r
    }
}

/// 按行排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
struct OnlyOneRightInRowInference;
impl Inference for OnlyOneRightInRowInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_rc().find_map(|vr| {
            vr.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        vr.iter()
                            .all(|p_iter| p_iter.rc.c == p.rc.c || !p_iter.drafts.is_contain(*v))
                    })
                    .and_then(|&ret| {
                        let cv = TheCoordsAndTheValue {
                            the_coords: p.coords,
                            the_value: vec![ret],
                        };
                        Some(InferenceResult {
                            inference: self,
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: make_simple_conclusion_when_set_value(
                                &field, &p.coords, ret,
                            ),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在 R{:?} 内唯一，因此 {:?} 只能填写 {:?}",
            Into::<RCCoords>::into(condition.the_coords),
            condition.the_value[0],
            Into::<RCCoords>::into(condition.the_coords).r + 1,
            Into::<RCCoords>::into(condition.the_coords),
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                    .collect::<Vec<String>>()
                    .join(" "),
                condition.the_value[0]
            ));
        }

        r
    }
}

/// 按列排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
struct OnlyOneRightInColInference;
impl Inference for OnlyOneRightInColInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_cr().find_map(|vc| {
            vc.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        vc.iter()
                            .all(|p_iter| p_iter.rc.r == p.rc.r || !p_iter.drafts.is_contain(*v))
                    })
                    .and_then(|&ret| {
                        let cv = TheCoordsAndTheValue {
                            the_coords: p.coords,
                            the_value: vec![ret],
                        };
                        Some(InferenceResult {
                            inference: self,
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: make_simple_conclusion_when_set_value(
                                &field, &p.coords, ret,
                            ),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在 C{:?} 内唯一，因此 {:?} 只能填写 {:?}",
            Into::<RCCoords>::into(condition.the_coords),
            condition.the_value[0],
            Into::<RCCoords>::into(condition.the_coords).c + 1,
            Into::<RCCoords>::into(condition.the_coords),
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                    .collect::<Vec<String>>()
                    .join(" "),
                condition.the_value[0]
            ));
        }

        r
    }
}

///  按宫排除法，每行中如果存在唯一草稿值，则填写该值，同时去除其余同一列宫的草稿值
struct OnlyOneRightInGridInference;
impl Inference for OnlyOneRightInGridInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_gn().find_map(|vg| {
            vg.iter().find_map(|&p| {
                p.drafts
                    .to_vec()
                    .iter()
                    .find(|&v| {
                        vg.iter()
                            .all(|p_iter| p_iter.gn.n == p.gn.n || !p_iter.drafts.is_contain(*v))
                    })
                    .and_then(|&ret| {
                        let cv = TheCoordsAndTheValue {
                            the_coords: p.coords,
                            the_value: vec![ret],
                        };
                        Some(InferenceResult {
                            inference: self,
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: make_simple_conclusion_when_set_value(
                                &field, &p.coords, ret,
                            ),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在 G{:?} 内唯一，因此 {:?} 只能填写 {:?}",
            Into::<GNCoords>::into(condition.the_coords),
            condition.the_value[0],
            Into::<GNCoords>::into(condition.the_coords).g + 1,
            Into::<GNCoords>::into(condition.the_coords),
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", Into::<GNCoords>::into(cv.the_coords)))
                    .collect::<Vec<String>>()
                    .join(" "),
                condition.the_value[0]
            ));
        }

        r
    }
}

/// 当一宫内的某种草稿值当且仅当在同一行时，可以排除该行内其余格子的该草稿值
struct RowUniqueDraftByGridExclusionInference;
impl Inference for RowUniqueDraftByGridExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_gn().find_map(|vg| {
            CellValue::iter().find_map(|v| {
                let cells_with_value = vg
                    .iter()
                    .filter(|&p| p.drafts.is_contain(v))
                    .collect::<Vec<_>>();

                if !cells_with_value.is_empty()
                    && cells_with_value
                        .iter()
                        .all(|&p| p.rc.r == cells_with_value[0].rc.r)
                {
                    let cells_in_same_row_but_not_in_same_grid: Vec<&Cell> = field
                        .collect_all_drafts_cells_in_r(cells_with_value[0].rc.r)
                        .into_iter()
                        .filter(|&p| p.gn.g != cells_with_value[0].gn.g && p.drafts.is_contain(v))
                        .collect();

                    if !cells_in_same_row_but_not_in_same_grid.is_empty() {
                        let condition = cells_with_value
                            .iter()
                            .map(|&p| TheCoordsAndTheValue {
                                the_coords: p.coords,
                                the_value: vec![v],
                            })
                            .collect();

                        let conclusion = cells_in_same_row_but_not_in_same_grid
                            .iter()
                            .map(|&p| TheCoordsAndTheValue {
                                the_coords: p.coords,
                                the_value: vec![v],
                            })
                            .collect();

                        Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{} 的所有可能 {:?} 在 G{:?} 内都只在 R{:?} 中，因此 ",
            inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<GNCoords>::into(cv.the_coords)))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value[0],
            Into::<GNCoords>::into(inference_result.condition[0].the_coords).g + 1,
            Into::<RCCoords>::into(inference_result.condition[0].the_coords).r + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();
            r.push_str(&format!(
                "{} 均不能填写 {:?}",
                removed_cells.join(" "),
                inference_result.condition[0].the_value[0]
            ));
        }

        r
    }
}

/// 当一宫内的某种草稿值当且仅当在同一列时，可以排除该列内其余格子的该草稿值
struct ColUniqueDraftByGridExclusionInference;
impl Inference for ColUniqueDraftByGridExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_gn().find_map(|vg| {
            CellValue::iter().find_map(|v| {
                let cells_with_value = vg
                    .iter()
                    .filter(|&p| p.drafts.is_contain(v))
                    .collect::<Vec<_>>();

                if !cells_with_value.is_empty()
                    && cells_with_value
                        .iter()
                        .all(|&p| p.rc.c == cells_with_value[0].rc.c)
                {
                    let cells_in_same_col_but_not_in_same_grid: Vec<&Cell> = field
                        .collect_all_drafts_cells_in_c(cells_with_value[0].rc.c)
                        .into_iter()
                        .filter(|&p| p.gn.g != cells_with_value[0].gn.g && p.drafts.is_contain(v))
                        .collect();

                    if !cells_in_same_col_but_not_in_same_grid.is_empty() {
                        let condition = cells_with_value
                            .iter()
                            .map(|&p| TheCoordsAndTheValue {
                                the_coords: p.coords,
                                the_value: vec![v],
                            })
                            .collect();

                        let conclusion = cells_in_same_col_but_not_in_same_grid
                            .iter()
                            .map(|&p| TheCoordsAndTheValue {
                                the_coords: p.coords,
                                the_value: vec![v],
                            })
                            .collect();

                        Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{} 的所有可能 {:?} 在 G{:?} 内都只在 C{:?} 中，因此 ",
            inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<GNCoords>::into(cv.the_coords)))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value[0],
            Into::<GNCoords>::into(inference_result.condition[0].the_coords).g + 1,
            Into::<RCCoords>::into(inference_result.condition[0].the_coords).c + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();
            r.push_str(&format!(
                "{} 均不能填写 {:?}",
                removed_cells.join(" "),
                inference_result.condition[0].the_value[0]
            ));
        }

        r
    }
}

/// 当一行的草稿数正好在一宫时，排除该宫的其他草稿数
struct GridUniqueDraftByRowExclusionInference;
impl Inference for GridUniqueDraftByRowExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_rc().find_map(|vr| {
            CellValue::iter().find_map(|v| {
                vr.iter()
                    .filter(|&p| p.drafts.is_contain(v))
                    .find(|&p| {
                        let vg = field.collect_all_drafts_cells_in_g(p.gn.g);
                        // 条件1：该行内的值都在同一个宫内
                        let all_in_same_grid = vr.iter().all(|&p_iter| p_iter.gn.g == p.gn.g);
                        // 条件2：第一个值所在的宫，在其他行内有值
                        let others_in_same_grid = vg
                            .iter()
                            .any(|&p_iter| p_iter.rc.r != p.rc.r && p_iter.drafts.is_contain(v));
                        all_in_same_grid && others_in_same_grid
                    })
                    .map(|p| {
                        let condition = vr
                            .iter()
                            .filter(|&p_iter| p_iter.drafts.is_contain(v))
                            .map(|p_iter| TheCoordsAndTheValue {
                                the_coords: p_iter.coords,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCoordsAndTheValue>>();

                        let conclusion = field
                            .collect_all_drafts_cells_in_g(p.gn.g)
                            .into_iter()
                            .filter(|p_iter| p_iter.rc.r != p.rc.r && p_iter.drafts.is_contain(v))
                            .map(|p_iter| TheCoordsAndTheValue {
                                the_coords: p_iter.coords,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCoordsAndTheValue>>();

                        InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        }
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{} 的所有可能 {:?} 在 R{:?} 内都只在 G{:?} 中，推导出：",
            inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value,
            Into::<RCCoords>::into(inference_result.condition[0].the_coords).r + 1,
            Into::<GNCoords>::into(inference_result.condition[0].the_coords).g + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();
            r.push_str(&format!(
                "{}均不能填写 {:?}",
                removed_cells.join(" "),
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 当一列的草稿数正好在一宫时，排除该宫的其他草稿数
struct GridUniqueDraftByColExclusionInference;
impl Inference for GridUniqueDraftByColExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_cr().find_map(|vc| {
            CellValue::iter().find_map(|v| {
                vc.iter()
                    .filter(|&p| p.drafts.is_contain(v))
                    .find(|&p| {
                        let vg = field.collect_all_drafts_cells_in_g(p.gn.g);
                        // 条件1：该行内的值都在同一个宫内
                        let all_in_same_grid = vc.iter().all(|&p_iter| p_iter.gn.g == p.gn.g);
                        // 条件2：第一个值所在的宫，在其他列内有值
                        let others_in_same_grid = vg
                            .iter()
                            .any(|&p_iter| p_iter.rc.c != p.rc.c && p_iter.drafts.is_contain(v));
                        all_in_same_grid && others_in_same_grid
                    })
                    .map(|p| {
                        let condition = vc
                            .iter()
                            .filter(|&p_iter| p_iter.drafts.is_contain(v))
                            .map(|p_iter| TheCoordsAndTheValue {
                                the_coords: p_iter.coords,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCoordsAndTheValue>>();

                        let conclusion = field
                            .collect_all_drafts_cells_in_g(p.gn.g)
                            .into_iter()
                            .filter(|p_iter| p_iter.rc.c != p.rc.c && p_iter.drafts.is_contain(v))
                            .map(|p_iter| TheCoordsAndTheValue {
                                the_coords: p_iter.coords,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCoordsAndTheValue>>();

                        InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        }
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{} 的所有可能 {:?} 在 C{:?} 内都只在 G{:?} 中，推导出：",
            inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value,
            Into::<RCCoords>::into(inference_result.condition[0].the_coords).c + 1,
            Into::<GNCoords>::into(inference_result.condition[0].the_coords).g + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();
            r.push_str(&format!(
                "{}均不能填写 {:?}",
                removed_cells.join(" "),
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 显性数对排除法（行），在某一行中，存在2/3/4数对时，排除该行中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是X，称之为【数对】，其中 2<=X<=4
struct RowExplicitNakedPairExclusionInference;
impl Inference for RowExplicitNakedPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        for vr in field.iter_all_drafts_cells_by_rc() {
            let mut all_combinations = Vec::new();
            for size in 2..=4 {
                let mut paths = Vec::new();
                crate::utils::generate_combinations(
                    vr.len(),
                    size,
                    0,
                    &mut paths,
                    &mut all_combinations,
                );
            }

            for (combo, rest) in all_combinations {
                let union_drafts: Drafts = combo
                    .iter()
                    .map(|&i| vr[i].drafts.clone())
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default();
                let union_drafts_vec = union_drafts.to_vec();
                // 检查并集的数量是否等于集合的数量
                if union_drafts_vec.len() == combo.len() {
                    let condition: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .map(|&i| TheCoordsAndTheValue {
                            the_coords: vr[i].coords,
                            the_value: union_drafts_vec.clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCoordsAndTheValue> = rest
                        .iter()
                        .filter_map(|&i| {
                            if vr[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| union_drafts_vec.contains(&val))
                            {
                                Some(TheCoordsAndTheValue {
                                    the_coords: vr[i].coords,
                                    the_value: union_drafts_vec.clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !conclusion.is_empty() {
                        return Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        });
                    }
                }
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let removed_values: Vec<String> = conclusion_remove_drafts[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            return format!(
                "{} 的草稿 {} 在同一 R{:?} 内形成了数对，因此该 R{:?} 内 {} 不能填写 {} ",
                condition_cells.join(" "),
                removed_values.join(" "),
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).r + 1,
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).r + 1,
                removed_cells.join(" "),
                removed_values.join(" ")
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// 显性数对排除法（列），在某一列中，存在2/3/4数对时，排除该列中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是X，称之为【数对】，其中 2<=X<=4
struct ColExplicitNakedPairExclusionInference;
impl Inference for ColExplicitNakedPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        for vc in field.iter_all_drafts_cells_by_cr() {
            let mut all_combinations = Vec::new();
            for size in 2..=4 {
                let mut paths = Vec::new();
                crate::utils::generate_combinations(
                    vc.len(),
                    size,
                    0,
                    &mut paths,
                    &mut all_combinations,
                );
            }

            for (combo, rest) in all_combinations {
                let union_drafts: Drafts = combo
                    .iter()
                    .map(|&i| vc[i].drafts.clone())
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default();
                let union_drafts_vec = union_drafts.to_vec();
                // 检查并集的数量是否等于集合的数量
                if union_drafts_vec.len() == combo.len() {
                    let condition: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .map(|&i| TheCoordsAndTheValue {
                            the_coords: vc[i].coords,
                            the_value: union_drafts_vec.clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCoordsAndTheValue> = rest
                        .iter()
                        .filter_map(|&i| {
                            if vc[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| union_drafts_vec.contains(&val))
                            {
                                Some(TheCoordsAndTheValue {
                                    the_coords: vc[i].coords,
                                    the_value: union_drafts_vec.clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !conclusion.is_empty() {
                        return Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        });
                    }
                }
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let removed_values: Vec<String> = conclusion_remove_drafts[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            return format!(
                "{} 的草稿 {} 在同一 C{:?} 内形成了数对，因此该 C{:?} 内 {} 不能填写 {} ",
                condition_cells.join(" "),
                removed_values.join(" "),
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).c + 1,
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).c + 1,
                removed_cells.join(" "),
                removed_values.join(" ")
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// 显性数对排除法（宫），在某一列中，存在2/3/4数对时，排除该列中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是X，称之为【数对】，其中 2<=X<=4
struct GridExplicitNakedPairExclusionInference;
impl Inference for GridExplicitNakedPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        for vg in field.iter_all_drafts_cells_by_gn() {
            let mut all_combinations = Vec::new();
            for size in 2..=4 {
                let mut paths = Vec::new();
                crate::utils::generate_combinations(
                    vg.len(),
                    size,
                    0,
                    &mut paths,
                    &mut all_combinations,
                );
            }

            for (combo, rest) in all_combinations {
                let union_drafts: Drafts = combo
                    .iter()
                    .map(|&i| vg[i].drafts.clone())
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default();
                let union_drafts_vec = union_drafts.to_vec();
                // 检查并集的数量是否等于集合的数量
                if union_drafts_vec.len() == combo.len() {
                    let condition: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .map(|&i| TheCoordsAndTheValue {
                            the_coords: vg[i].coords,
                            the_value: union_drafts_vec.clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCoordsAndTheValue> = rest
                        .iter()
                        .filter_map(|&i| {
                            if vg[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| union_drafts_vec.contains(&val))
                            {
                                Some(TheCoordsAndTheValue {
                                    the_coords: vg[i].coords,
                                    the_value: union_drafts_vec.clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !conclusion.is_empty() {
                        return Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        });
                    }
                }
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<GNCoords>::into(cv.the_coords)))
                .collect();

            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<GNCoords>::into(cv.the_coords)))
                .collect();

            let removed_values: Vec<String> = conclusion_remove_drafts[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            return format!(
                "{} 的草稿 {} 在同一 G{:?} 内形成了数对，因此该 G{:?} 内 {} 不能填写 {} ",
                condition_cells.join(" "),
                removed_values.join(" "),
                Into::<GNCoords>::into(conclusion_remove_drafts[0].the_coords).g + 1,
                Into::<GNCoords>::into(conclusion_remove_drafts[0].the_coords).g + 1,
                removed_cells.join(" "),
                removed_values.join(" ")
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// 隐性数对排除法（行），在某一行中，存在2/3/4数对时，排除该行中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是总候选数-X，则称剩余候选数组成的集合为【数对】，其中 2<=X<=4
struct RowExplicitHiddenPairExclusionInference;
impl Inference for RowExplicitHiddenPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        for vr in field.iter_all_drafts_cells_by_rc() {
            let mut all_combinations = Vec::new();
            for size in 2..=4 {
                let mut paths = Vec::new();
                crate::utils::generate_combinations(
                    vr.len(),
                    size,
                    0,
                    &mut paths,
                    &mut all_combinations,
                );
            }

            for (combo, rest) in all_combinations {
                let rest_union_drafts: Drafts = rest
                    .iter()
                    .map(|&i| vr[i].drafts.clone())
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default();
                let rest_union_drafts_vec = rest_union_drafts.to_vec();
                // 检查剩余的并集数量是否等于整个候选数-组合的数量
                if rest_union_drafts_vec.len() == vr.len() - combo.len() {
                    let combo_union_drafts = combo
                        .iter()
                        .map(|&i| vr[i].drafts.clone())
                        .reduce(|a, b| a.union(b))
                        .unwrap_or_default();
                    let hidden_pair_drafts = combo_union_drafts.subtract(rest_union_drafts);
                    let condition: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .map(|&i| TheCoordsAndTheValue {
                            the_coords: vr[i].coords,
                            the_value: hidden_pair_drafts.to_vec().clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .filter_map(|&i| {
                            if vr[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| rest_union_drafts_vec.contains(&val))
                            {
                                Some(TheCoordsAndTheValue {
                                    the_coords: vr[i].coords,
                                    the_value: rest_union_drafts.to_vec().clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !conclusion.is_empty() {
                        return Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        });
                    }
                }
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let condition_values: Vec<String> = inference_result.condition[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            let removed_values: Vec<String> = conclusion_remove_drafts[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            return format!(
                "{} 在 R{:?} 内形成了隐性数对 {} ，因此该 R{:?} 内 {} 不能填写 {} ",
                condition_cells.join(" "),
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).r + 1,
                condition_values.join(" "),
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).r + 1,
                condition_cells.join(" "),
                removed_values.join(" ")
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// 隐性数对排除法（列），在某一列中，存在2/3/4数对时，排除该行中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是总候选数-X，则称剩余候选数组成的集合为【数对】，其中 2<=X<=4
struct ColExplicitHiddenPairExclusionInference;
impl Inference for ColExplicitHiddenPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        for vc in field.iter_all_drafts_cells_by_cr() {
            let mut all_combinations = Vec::new();
            for size in 2..=4 {
                let mut paths = Vec::new();
                crate::utils::generate_combinations(
                    vc.len(),
                    size,
                    0,
                    &mut paths,
                    &mut all_combinations,
                );
            }

            for (combo, rest) in all_combinations {
                let rest_union_drafts: Drafts = rest
                    .iter()
                    .map(|&i| vc[i].drafts.clone())
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default();
                let rest_union_drafts_vec = rest_union_drafts.to_vec();
                // 检查剩余的并集数量是否等于整个候选数-组合的数量
                if rest_union_drafts_vec.len() == vc.len() - combo.len() {
                    let combo_union_drafts = combo
                        .iter()
                        .map(|&i| vc[i].drafts.clone())
                        .reduce(|a, b| a.union(b))
                        .unwrap_or_default();
                    let hidden_pair_drafts = combo_union_drafts.subtract(rest_union_drafts);
                    let condition: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .map(|&i| TheCoordsAndTheValue {
                            the_coords: vc[i].coords,
                            the_value: hidden_pair_drafts.to_vec().clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .filter_map(|&i| {
                            if vc[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| rest_union_drafts_vec.contains(&val))
                            {
                                Some(TheCoordsAndTheValue {
                                    the_coords: vc[i].coords,
                                    the_value: rest_union_drafts.to_vec().clone(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !conclusion.is_empty() {
                        return Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        });
                    }
                }
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let condition_values: Vec<String> = inference_result.condition[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            let removed_values: Vec<String> = conclusion_remove_drafts[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            return format!(
                "{} 在 C{:?} 内形成了隐性数对 {} ，因此该 C{:?} 内 {} 不能填写 {} ",
                condition_cells.join(" "),
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).c + 1,
                condition_values.join(" "),
                Into::<RCCoords>::into(conclusion_remove_drafts[0].the_coords).c + 1,
                condition_cells.join(" "),
                removed_values.join(" ")
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// 隐性数对排除法（宫），在某一宫中，存在2/3/4数对时，排除该行中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是总候选数-X，则称剩余候选数组成的集合为【数对】，其中 2<=X<=4
struct GridExplicitHiddenPairExclusionInference;
impl Inference for GridExplicitHiddenPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        for vg in field.iter_all_drafts_cells_by_gn() {
            let all_combinations = (2..=4)
                .flat_map(|size| {
                    let mut paths = Vec::new();
                    let mut combinations = Vec::new();
                    crate::utils::generate_combinations(
                        vg.len(),
                        size,
                        0,
                        &mut paths,
                        &mut combinations,
                    );
                    combinations
                })
                .collect::<Vec<_>>();

            for (combo, rest) in all_combinations {
                let rest_union_drafts: Drafts = rest
                    .iter()
                    .map(|&i| vg[i].drafts.clone())
                    .reduce(|a, b| a.union(b))
                    .unwrap_or_default();
                let rest_union_drafts_vec = rest_union_drafts.to_vec();
                // 检查剩余的并集数量是否等于整个候选数-组合的数量
                if rest_union_drafts_vec.len() == vg.len() - combo.len() {
                    let combo_union_drafts = combo
                        .iter()
                        .map(|&i| vg[i].drafts.clone())
                        .reduce(|a, b| a.union(b))
                        .unwrap_or_default();
                    let hidden_pair_drafts = combo_union_drafts.subtract(rest_union_drafts);
                    let condition: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .map(|&i| TheCoordsAndTheValue {
                            the_coords: vg[i].coords,
                            the_value: hidden_pair_drafts.to_vec(),
                        })
                        .collect();
                    let conclusion: Vec<TheCoordsAndTheValue> = combo
                        .iter()
                        .filter_map(|&i| {
                            if vg[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| rest_union_drafts_vec.contains(&val))
                            {
                                Some(TheCoordsAndTheValue {
                                    the_coords: vg[i].coords,
                                    the_value: rest_union_drafts.to_vec(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !conclusion.is_empty() {
                        return Some(InferenceResult {
                            inference: self,
                            condition,
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(conclusion),
                        });
                    }
                }
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let condition_values: Vec<String> = inference_result.condition[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            let removed_values: Vec<String> = conclusion_remove_drafts[0]
                .the_value
                .iter()
                .map(|cv| format!("{:?}", cv))
                .collect();

            return format!(
                "{} 在 G{:?} 内形成了隐性数对 {} ，因此该 G{:?} 内 {} 不能填写 {} ",
                condition_cells.join(" "),
                Into::<GNCoords>::into(conclusion_remove_drafts[0].the_coords).g + 1,
                condition_values.join(" "),
                Into::<GNCoords>::into(conclusion_remove_drafts[0].the_coords).g + 1,
                condition_cells.join(" "),
                removed_values.join(" ")
            );
        } else {
            String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
        }
    }
}

/// n阶Fish，在一个维度（行/列）中，某个数字只出现在n个单元格中，且正好有n-1个维度的单元格正好位于相同的另一个列中（允许残缺，不允许多）
struct NStepFishInference;
impl Inference for NStepFishInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        // 构造返回条件
        fn create_condition(
            v: CellValue,
            direction: &IterDirection,
            one_indexes: &[usize],   // 阶数个usize的数组
            other_indexes: &[usize], // 阶数个usize的数组
        ) -> Vec<TheCoordsAndTheValue> {
            let mut condition = Vec::new();

            // 确保coords1和coords2的长度等于阶数
            let n = one_indexes.len();
            assert!(n == other_indexes.len());

            // 生成所有可能的coords1和coords2的组合
            for i in 0..n {
                for j in 0..n {
                    let (one_index, other_index) = (one_indexes[i], other_indexes[j]);
                    condition.push(create_simple_cell_and_value(
                        get_coords_with_direction(one_index, other_index, direction),
                        v,
                    ));
                }
            }

            condition
        }

        // 构造返回结论
        fn create_conclusion<'a>(
            field: &'a Sudoku,
            v: CellValue,
            direction: &'a IterDirection,
            one_indexes: &[usize],   // 阶数个usize的数组
            other_indexes: &[usize], // 阶数个usize的数组
        ) -> Vec<TheCoordsAndTheValue> {
            let mut conclusion = Vec::new();

            // 确保one_indexes和other_indexes的长度等于阶数
            let n = one_indexes.len();
            assert!(n == other_indexes.len());

            // 检查每个one_index中的每个other_index对应的单元格
            for one_index in 0..9 {
                // 确保不是同一个第一维度
                if !one_indexes.contains(&one_index) {
                    for &other_index in other_indexes {
                        let rc = get_rc_coord_with_direction(one_index, other_index, direction);
                        let cell = field.get_cell_ref_by_rc(rc);
                        if cell.status == CellStatus::DRAFT && cell.drafts.is_contain(v) {
                            conclusion.push(create_simple_cell_and_value(rc.into(), v));
                        }
                    }
                }
            }

            conclusion
        }

        // 判断是否满足数对条件：数组不为空，且待判断数组的每一个值，均在原数组中
        fn is_n_fish_pair(v1: &Vec<(usize, usize)>, v2: &Vec<(usize, usize)>) -> bool {
            if v2.is_empty() {
                false
            } else {
                v2.iter()
                    .all(|&(_, v2_c)| v1.iter().any(|&(_, v1_c)| v2_c == v1_c))
            }
        }

        // 返回值是某个遍历维度下，所有满足该行/列中只有两个value的坐标
        fn self_analyze_with_direction<'a>(
            inference: &'a dyn Inference,
            field: &'a Sudoku,
            v: CellValue,
            direction: &'a IterDirection,
        ) -> Option<InferenceResult<'a>> {
            let mut all_v_in_field: Vec<Vec<(usize, usize)>> = Vec::new();
            // 这里先求出草稿v的在某个维度上的分布
            for one_index in 0..9 {
                let mut all_v_in_one_index = Vec::new();
                for other_index in 0..9 {
                    let p = field.get_cell_ref_by_rc(get_rc_coord_with_direction(
                        one_index,
                        other_index,
                        &direction,
                    ));
                    if p.status == CellStatus::DRAFT && p.drafts.is_contain(v) {
                        all_v_in_one_index.push(match &direction {
                            IterDirection::Row => (p.rc.r, p.rc.c),
                            IterDirection::Column => (p.rc.c, p.rc.r),
                            IterDirection::Grid => todo!(),
                        });
                    }
                }
                all_v_in_field.push(all_v_in_one_index);
            }
            for one_index in 0..9 {
                let cur_len = all_v_in_field[one_index].len();
                if cur_len >= 2 && cur_len <= 4 {
                    // 这里要找cur_len-1个is_n_fish_pair为true的行出来
                    let mut pair_one_index = vec![one_index];
                    for one_index_2 in 0..9 {
                        if one_index_2 != one_index
                            && is_n_fish_pair(
                                &all_v_in_field[one_index],
                                &all_v_in_field[one_index_2],
                            )
                        {
                            pair_one_index.push(one_index_2);
                        }
                    }
                    if pair_one_index.len() == cur_len {
                        let condition = create_condition(
                            v,
                            direction,
                            &pair_one_index,
                            &all_v_in_field[one_index]
                                .iter()
                                .map(|(_, other_index)| *other_index)
                                .collect::<Vec<usize>>(),
                        );
                        let conclusion = create_conclusion(
                            field,
                            v,
                            direction,
                            &pair_one_index,
                            &all_v_in_field[one_index]
                                .iter()
                                .map(|(_, other_index)| *other_index)
                                .collect::<Vec<usize>>(),
                        );
                        if !conclusion.is_empty() {
                            return Some(InferenceResult {
                                inference,
                                condition,
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(conclusion),
                            });
                        }
                    }
                }
            }

            None
        }

        CellValue::iter().find_map(|v| {
            self_analyze_with_direction(self, &field, v, &IterDirection::Row).or(
                self_analyze_with_direction(self, &field, v, &IterDirection::Column),
            )
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let condition_cells: Vec<String> = inference_result
                .condition
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let conclusion_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", Into::<RCCoords>::into(cv.the_coords)))
                .collect();

            let fish_step = match condition_cells.len() {
                3..=4 => "二",
                5..=9 => "三",
                10..=16 => "四",
                _ => "未知",
            };

            return format!(
                "{} 形成了 {}阶鱼 ，因此 {} 不能填写 {:?} ",
                condition_cells.join(" "),
                fish_step,
                conclusion_cells.join(" "),
                inference_result.condition[0].the_value[0]
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// 暴力破解法，以上所有策略都失效的情况下，使用这个方法破解数独，计算机直接强行计算
/// 如果数独存在多解，也返回None
struct ExploitInference;
impl Inference for ExploitInference {
    fn analyze<'a>(&'a self, field: &'a Sudoku) -> Option<InferenceResult<'a>> {
        let solve_field = field.sovle();

        if solve_field.is_empty() {
            None
        } else {
            if solve_field.len() == 1 {
                let solve_field = &solve_field[0];
                let mut conclusion = Vec::new();

                for r in 0..9 {
                    for c in 0..9 {
                        let rc = RCCoords { r, c };
                        let p1 = solve_field.get_cell_ref_by_rc(rc);
                        let p2 = field.get_cell_ref_by_rc(rc);
                        if p1.status != p2.status {
                            conclusion.push(TheCoordsAndTheValue {
                                the_coords: p2.coords,
                                the_value: vec![p1.value],
                            });
                        }
                    }
                }

                Some(InferenceResult {
                    inference: self,
                    condition: vec![],
                    conclusion_set_value: Some(conclusion),
                    conclusion_remove_drafts: None,
                })
            } else {
                // 多解数独，返回
                None
            }
        }
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        "暴力破解法".to_string()
    }
}
