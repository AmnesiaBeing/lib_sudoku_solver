use crate::{types::{Cell, CellStatus, CellValue, Coords, Drafts, Field, TheCellAndTheValue}, utils::generate_combinations};

pub struct InferenceResult<'a> {
    inference: &'a dyn Inference,
    condition: Vec<TheCellAndTheValue<'a>>,
    conclusion_set_value: Option<Vec<TheCellAndTheValue<'a>>>,
    conclusion_remove_drafts: Option<Vec<TheCellAndTheValue<'a>>>,
}

trait Inference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>>;
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
            ],
        }
    }

    pub fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        self.inferences.iter().find_map(|inf| inf.analyze(field))
    }

    pub fn apply(field: &Field, result: InferenceResult) -> Field {
        let mut ret = field.clone();
        if result.conclusion_set_value.is_some() {
            result.conclusion_set_value.unwrap().iter().for_each(|cv| {
                let p = ret.get_cell_mut_by_coords(Coords::RC((cv.the_cell).rc));
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
                    let p = ret.get_cell_mut_by_coords(Coords::RC((cv.the_cell).rc));
                    cv.the_value.iter().for_each(|&v| p.drafts.remove_draft(v));
                })
        }
        ret
    }
}

impl std::fmt::Debug for InferenceResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inference.write_result(&self))
    }
}

/// 唯余法，遍历所有草稿单元格，如果存在唯一草稿，则说明这个草稿填写该数字
#[derive(Clone)]
struct OnlyOneLeftInference;
impl Inference for OnlyOneLeftInference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        field.collect_all_drafts_cells().iter().find_map(|&p| {
            p.drafts.try_get_the_only_one().map(|cv| {
                let cell_and_value = TheCellAndTheValue {
                    the_cell: p,
                    the_value: vec![cv],
                };
                InferenceResult {
                    inference: self,
                    condition: vec![cell_and_value.clone()],
                    conclusion_set_value: Some(vec![cell_and_value.clone()]),
                    conclusion_remove_drafts: field
                        .collect_all_drafts_coords_by_the_coords_and_the_value(
                            cell_and_value.the_cell,
                            cell_and_value.the_value[0],
                        ),
                }
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在格内唯一，因此 {:?} 只能填写 {:?} ",
            condition.the_cell.rc,
            condition.the_value[0],
            condition.the_cell.rc,
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                        let cv = TheCellAndTheValue {
                            the_cell: p,
                            the_value: vec![ret],
                        };
                        Some(InferenceResult {
                            inference: self,
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: field
                                .collect_all_drafts_coords_by_the_coords_and_the_value(p, ret),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在 R{:?} 内唯一，因此 {:?} 只能填写 {:?}",
            condition.the_cell.rc,
            condition.the_value[0],
            condition.the_cell.rc.r + 1,
            condition.the_cell.rc,
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                        let cv = TheCellAndTheValue {
                            the_cell: p,
                            the_value: vec![ret],
                        };
                        Some(InferenceResult {
                            inference: self,
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: field
                                .collect_all_drafts_coords_by_the_coords_and_the_value(p, ret),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在 C{:?} 内唯一，因此 {:?} 只能填写 {:?}",
            condition.the_cell.rc,
            condition.the_value[0],
            condition.the_cell.rc.c + 1,
            condition.the_cell.rc,
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                        let cv = TheCellAndTheValue {
                            the_cell: p,
                            the_value: vec![ret],
                        };
                        Some(InferenceResult {
                            inference: self,
                            condition: vec![cv.clone()],
                            conclusion_set_value: Some(vec![cv]),
                            conclusion_remove_drafts: field
                                .collect_all_drafts_coords_by_the_coords_and_the_value(p, ret),
                        })
                    })
            })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let condition = &inference_result.condition[0];
        let mut r = format!(
            "{:?} 的可能 {:?} 在 G{:?} 内唯一，因此 {:?} 只能填写 {:?}",
            condition.the_cell.gn,
            condition.the_value[0],
            condition.the_cell.gn.g + 1,
            condition.the_cell.gn,
            condition.the_value[0]
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            r.push_str(&format!(
                "，并移除 {} 的可能 {:?}",
                conclusion_remove_drafts
                    .iter()
                    .map(|cv| format!("{:?}", cv.the_cell.gn))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                            .map(|&p| TheCellAndTheValue {
                                the_cell: p,
                                the_value: vec![v],
                            })
                            .collect();

                        let conclusion = cells_in_same_row_but_not_in_same_grid
                            .iter()
                            .map(|&p| TheCellAndTheValue {
                                the_cell: p,
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
                .map(|cv| format!("{:?}", cv.the_cell.gn))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value[0],
            inference_result.condition[0].the_cell.gn.g + 1,
            inference_result.condition[0].the_cell.rc.r + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                            .map(|&p| TheCellAndTheValue {
                                the_cell: p,
                                the_value: vec![v],
                            })
                            .collect();

                        let conclusion = cells_in_same_col_but_not_in_same_grid
                            .iter()
                            .map(|&p| TheCellAndTheValue {
                                the_cell: p,
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
                .map(|cv| format!("{:?}", cv.the_cell.gn))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value[0],
            inference_result.condition[0].the_cell.gn.g + 1,
            inference_result.condition[0].the_cell.rc.c + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                            .map(|p_iter| TheCellAndTheValue {
                                the_cell: p_iter,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCellAndTheValue>>();

                        let conclusion = field
                            .collect_all_drafts_cells_in_g(p.gn.g)
                            .into_iter()
                            .filter(|p_iter| p_iter.rc.r != p.rc.r && p_iter.drafts.is_contain(v))
                            .map(|p_iter| TheCellAndTheValue {
                                the_cell: p_iter,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCellAndTheValue>>();

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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.r + 1,
            inference_result.condition[0].the_cell.gn.g + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                            .map(|p_iter| TheCellAndTheValue {
                                the_cell: p_iter,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCellAndTheValue>>();

                        let conclusion = field
                            .collect_all_drafts_cells_in_g(p.gn.g)
                            .into_iter()
                            .filter(|p_iter| p_iter.rc.c != p.rc.c && p_iter.drafts.is_contain(v))
                            .map(|p_iter| TheCellAndTheValue {
                                the_cell: p_iter,
                                the_value: vec![v],
                            })
                            .collect::<Vec<TheCellAndTheValue>>();

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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
                .collect::<Vec<String>>()
                .join(" "),
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.c + 1,
            inference_result.condition[0].the_cell.gn.g + 1
        );

        if let Some(conclusion_remove_drafts) = &inference_result.conclusion_remove_drafts {
            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                    let condition: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .map(|&i| TheCellAndTheValue {
                            the_cell: &vr[i],
                            the_value: union_drafts_vec.clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCellAndTheValue<'_>> = rest
                        .iter()
                        .filter_map(|&i| {
                            if vr[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| union_drafts_vec.contains(&val))
                            {
                                Some(TheCellAndTheValue {
                                    the_cell: &vr[i],
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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
                .collect();

            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
                conclusion_remove_drafts[0].the_cell.rc.r + 1,
                conclusion_remove_drafts[0].the_cell.rc.r + 1,
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                    let condition: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .map(|&i| TheCellAndTheValue {
                            the_cell: &vc[i],
                            the_value: union_drafts_vec.clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCellAndTheValue<'_>> = rest
                        .iter()
                        .filter_map(|&i| {
                            if vc[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| union_drafts_vec.contains(&val))
                            {
                                Some(TheCellAndTheValue {
                                    the_cell: &vc[i],
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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
                .collect();

            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
                conclusion_remove_drafts[0].the_cell.rc.c + 1,
                conclusion_remove_drafts[0].the_cell.rc.c + 1,
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                    let condition: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .map(|&i| TheCellAndTheValue {
                            the_cell: &vg[i],
                            the_value: union_drafts_vec.clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCellAndTheValue<'_>> = rest
                        .iter()
                        .filter_map(|&i| {
                            if vg[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| union_drafts_vec.contains(&val))
                            {
                                Some(TheCellAndTheValue {
                                    the_cell: &vg[i],
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
                .map(|cv| format!("{:?}", cv.the_cell.gn))
                .collect();

            let removed_cells: Vec<String> = conclusion_remove_drafts
                .iter()
                .map(|cv| format!("{:?}", cv.the_cell.gn))
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
                conclusion_remove_drafts[0].the_cell.gn.g + 1,
                conclusion_remove_drafts[0].the_cell.gn.g + 1,
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                    let condition: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .map(|&i| TheCellAndTheValue {
                            the_cell: &vr[i],
                            the_value: hidden_pair_drafts.to_vec().clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .filter_map(|&i| {
                            if vr[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| rest_union_drafts_vec.contains(&val))
                            {
                                Some(TheCellAndTheValue {
                                    the_cell: &vr[i],
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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
                conclusion_remove_drafts[0].the_cell.rc.r + 1,
                condition_values.join(" "),
                conclusion_remove_drafts[0].the_cell.rc.r + 1,
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                    let condition: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .map(|&i| TheCellAndTheValue {
                            the_cell: &vc[i],
                            the_value: hidden_pair_drafts.to_vec().clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .filter_map(|&i| {
                            if vc[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| rest_union_drafts_vec.contains(&val))
                            {
                                Some(TheCellAndTheValue {
                                    the_cell: &vc[i],
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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
                conclusion_remove_drafts[0].the_cell.rc.c + 1,
                condition_values.join(" "),
                conclusion_remove_drafts[0].the_cell.rc.c + 1,
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
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
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
                    let condition: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .map(|&i| TheCellAndTheValue {
                            the_cell: &vg[i],
                            the_value: hidden_pair_drafts.to_vec().clone(),
                        })
                        .collect();
                    let conclusion: Vec<TheCellAndTheValue<'_>> = combo
                        .iter()
                        .filter_map(|&i| {
                            if vg[i]
                                .drafts
                                .to_vec()
                                .iter()
                                .any(|&val| rest_union_drafts_vec.contains(&val))
                            {
                                Some(TheCellAndTheValue {
                                    the_cell: &vg[i],
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
                .map(|cv| format!("{:?}", cv.the_cell.rc))
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
                conclusion_remove_drafts[0].the_cell.rc.c + 1,
                condition_values.join(" "),
                conclusion_remove_drafts[0].the_cell.rc.c + 1,
                condition_cells.join(" "),
                removed_values.join(" ")
            );
        }

        String::new() // 如果没有结论，返回一个空字符串，正常情况下，不应该到这里来
    }
}

/// X-Wings，鱼
/// X（2<=X<=4）行的某个值可能所在的位置的交集的长度，正好也为X，说明形成了正交，可排除列中的其他格子的草稿数（反过来也同理）
struct XWingsInference;
impl Inference for XWingsInference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        let mut all_combinations = Vec::new();
        for size in 2..=4 {
            let mut paths = Vec::new();
            crate::utils::generate_combinations(
                9,
                size,
                0,
                &mut paths,
                &mut all_combinations,
            );
        }

        // 先按行视角来看
        

        todo!()
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        todo!()
    }
}
