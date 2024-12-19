use std::collections::HashMap;

use crate::types::{
    Cell, CellStatus, CellValue, Coords, Drafts, Field, GNCoords, RCCoords, TheCellAndTheValue,
};

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
                Box::new(RowUniqueDraftByGridInference),
                Box::new(ColUniqueDraftByGridExclusionInference),
                Box::new(BoxUniqueDraftByRowExclusionInference),
                Box::new(RowExplicitPairExclusionInference),
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
        field
            .collect_all_drafts_cells()
            .iter()
            .find_map(|&p| {
                (*p).drafts.try_get_the_only_one().and_then(|cv| {
                    Some(TheCellAndTheValue {
                        the_cell: p,
                        the_value: vec![cv],
                    })
                })
            })
            .and_then(move |ret| {
                Some({
                    InferenceResult {
                        inference: self,
                        condition: vec![ret.clone()],
                        conclusion_set_value: Some(vec![ret.clone()]),
                        conclusion_remove_drafts: field
                            .collect_all_drafts_coords_by_the_coords_and_the_value(
                                ret.the_cell,
                                ret.the_value[0],
                            ),
                    }
                })
            })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r = format!(
            "{:?} 的可能 {:?} 在格内唯一，推导出：这里只能填写 {:?} ",
            inference_result.condition[0].the_cell.rc,
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_value
        );

        if inference_result.conclusion_remove_drafts.is_some() {
            r.push_str(&format!("，且移除 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "的可能 {:?} ",
                inference_result.condition[0].the_value
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
                        !(vr.iter().fold(false, |acc, p_iter| {
                            acc || ((p_iter.rc.c != p.rc.c) && (p_iter.drafts.is_contain(*v)))
                        }))
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
        let mut r = format!(
            "{:?} 的可能 {:?} 在 R{:?} 内唯一，推导出：这里只能填写 {:?} ",
            inference_result.condition[0].the_cell.rc,
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.r + 1,
            inference_result.condition[0].the_value
        );

        if inference_result.conclusion_remove_drafts.is_some() {
            r.push_str(&format!("，且移除 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "的可能 {:?} ",
                inference_result.condition[0].the_value
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
                        !(vc.iter().fold(false, |acc, p_iter| {
                            acc || ((p_iter.rc.r != p.rc.r) && (p_iter.drafts.is_contain(*v)))
                        }))
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
        let mut r = format!(
            "{:?} 的可能 {:?} 在 C{:?} 内唯一，推导出：这里只能填写 {:?} ",
            inference_result.condition[0].the_cell.rc,
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.c + 1,
            inference_result.condition[0].the_value
        );

        if inference_result.conclusion_remove_drafts.is_some() {
            r.push_str(&format!("，且移除 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "的可能 {:?} ",
                inference_result.condition[0].the_value
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
                        !(vg.iter().fold(false, |acc, p_iter| {
                            acc || ((p_iter.gn.n != p.gn.n) && (p_iter.drafts.is_contain(*v)))
                        }))
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
        let mut r = format!(
            "{:?} 的可能 {:?} 在 G{:?} 内唯一，推导出：这里只能填写 {:?} ",
            inference_result.condition[0].the_cell.gn,
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.gn.g + 1,
            inference_result.condition[0].the_value
        );

        if inference_result.conclusion_remove_drafts.is_some() {
            r.push_str(&format!("，且移除 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.gn));
                });
            r.push_str(&format!(
                "的可能 {:?} ",
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 当一宫内的某种草稿值当且仅当在同一行时，可以排除该行内其余格子的该草稿值
struct RowUniqueDraftByGridInference;
impl Inference for RowUniqueDraftByGridInference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_gn().find_map(|vg| {
            CellValue::iter()
                .filter_map(|v| {
                    let tmp: Vec<&Cell> = vg
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vr = field.collect_all_drafts_cells_in_r(vp[0].rc.r);
                        // 条件1：该宫内其他行没有这个值
                        let ret1 = !vp.iter().any(|&p| (p.rc.r != vp[0].rc.r));
                        // 条件2：宫外该行内有这个值
                        let ret2 = vr
                            .iter()
                            .filter_map(|&vr_p_iter| {
                                ((vr_p_iter.gn.g != vp[0].gn.g) && vr_p_iter.drafts.is_contain(v))
                                    .then_some(vr_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        if ret1 && ret2.len() != 0 {
                            Some(InferenceResult {
                                inference: self,
                                condition: vp
                                    .iter()
                                    .map(|&p| TheCellAndTheValue {
                                        the_cell: p,
                                        the_value: vec![v],
                                    })
                                    .collect::<Vec<TheCellAndTheValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| TheCellAndTheValue {
                                            the_cell: p,
                                            the_value: vec![v],
                                        })
                                        .collect::<Vec<TheCellAndTheValue>>(),
                                ),
                            })
                        } else {
                            None
                        }
                    }
                })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r: String = "".to_string();
        inference_result.condition.iter().for_each(|cv| {
            r.push_str(&format!("{:?} ", cv.the_cell.rc));
        });
        r.push_str(&format!(
            "的所有可能 {:?} 在 G{:?} 内都只在 R{:?} 中，推导出：",
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.gn.g + 1,
            inference_result.condition[0].the_cell.rc.r + 1
        ));

        if inference_result.conclusion_remove_drafts.is_some() {
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "均不能填写 {:?} 。",
                inference_result.condition[0].the_value
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
            CellValue::iter()
                .filter_map(|v| {
                    let tmp: Vec<&Cell> = vg
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vc = field.collect_all_drafts_cells_in_c(vp[0].rc.c);
                        // 条件1：该宫内其他列没有这个值
                        let ret1 = !vp.iter().any(|&p| (p.rc.c != vp[0].rc.c));
                        // 条件2：宫外该列内有这个值
                        let ret2 = vc
                            .iter()
                            .filter_map(|&vc_p_iter| {
                                ((vc_p_iter.gn.g != vp[0].gn.g) && vc_p_iter.drafts.is_contain(v))
                                    .then_some(vc_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        (ret1 && ret2.len() != 0).then_some(InferenceResult {
                            inference: self,
                            condition: vp
                                .iter()
                                .map(|&p| TheCellAndTheValue {
                                    the_cell: p,
                                    the_value: vec![v],
                                })
                                .collect::<Vec<TheCellAndTheValue>>(),
                            conclusion_set_value: None,
                            conclusion_remove_drafts: Some(
                                ret2.iter()
                                    .map(|&p| TheCellAndTheValue {
                                        the_cell: p,
                                        the_value: vec![v],
                                    })
                                    .collect::<Vec<TheCellAndTheValue>>(),
                            ),
                        })
                    }
                })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r: String = "".to_string();
        inference_result.condition.iter().for_each(|cv| {
            r.push_str(&format!("{:?} ", cv.the_cell.rc));
        });
        r.push_str(&format!(
            "的所有可能 {:?} 在 G{:?} 内都只在 C{:?} 中，推导出：",
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.gn.g + 1,
            inference_result.condition[0].the_cell.rc.c + 1
        ));

        if inference_result.conclusion_remove_drafts.is_some() {
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "均不能填写 {:?} 。",
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 当一行的草稿数正好在一宫时，排除该宫的其他草稿数
struct BoxUniqueDraftByRowExclusionInference;
impl Inference for BoxUniqueDraftByRowExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_rc().find_map(|vr| {
            CellValue::iter()
                .filter_map(|v| {
                    let tmp: Vec<&Cell> = vr
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vg = field.collect_all_drafts_cells_in_g(vp[0].gn.g);
                        // 条件1：该行内的值都在同一个宫内
                        let ret1 = !vp.iter().any(|&p| (p.gn.g != vp[0].gn.g));
                        // 条件2：第一个值所在的宫，在其他行内有值
                        let ret2 = vg
                            .iter()
                            .filter_map(|&vr_p_iter| {
                                ((vr_p_iter.rc.r != vp[0].rc.r) && vr_p_iter.drafts.is_contain(v))
                                    .then_some(vr_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        if ret1 && ret2.len() != 0 {
                            Some(InferenceResult {
                                inference: self,
                                condition: vp
                                    .iter()
                                    .map(|&p| TheCellAndTheValue {
                                        the_cell: p,
                                        the_value: vec![v],
                                    })
                                    .collect::<Vec<TheCellAndTheValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| TheCellAndTheValue {
                                            the_cell: p,
                                            the_value: vec![v],
                                        })
                                        .collect::<Vec<TheCellAndTheValue>>(),
                                ),
                            })
                        } else {
                            None
                        }
                    }
                })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r: String = "".to_string();
        inference_result.condition.iter().for_each(|cv| {
            r.push_str(&format!("{:?} ", cv.the_cell.rc));
        });
        r.push_str(&format!(
            "的所有可能 {:?} 在 R{:?} 内都只在 G{:?} 中，推导出：",
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.c + 1,
            inference_result.condition[0].the_cell.gn.g + 1
        ));

        if inference_result.conclusion_remove_drafts.is_some() {
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "均不能填写 {:?} 。",
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 当一列的草稿数正好在一宫时，排除该宫的其他草稿数
struct BoxUniqueDraftByColExclusionInference;
impl Inference for BoxUniqueDraftByColExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        field.iter_all_drafts_cells_by_cr().find_map(|vc| {
            CellValue::iter()
                .filter_map(|v| {
                    let tmp: Vec<&Cell> = vc
                        .iter()
                        .filter_map(|&p| p.drafts.is_contain(v).then_some(p))
                        .collect::<Vec<&Cell>>();
                    (tmp.len() != 0).then_some((v, tmp))
                })
                .find_map(|(v, vp)| {
                    {
                        let vg = field.collect_all_drafts_cells_in_g(vp[0].gn.g);
                        // 条件1：该列内的值都在同一个宫内
                        let ret1 = !vp.iter().any(|&p| (p.gn.g != vp[0].gn.g));
                        // 条件2：第一个值所在的宫，在其他列内有值
                        let ret2 = vg
                            .iter()
                            .filter_map(|&vr_p_iter| {
                                ((vr_p_iter.rc.c != vp[0].rc.c) && vr_p_iter.drafts.is_contain(v))
                                    .then_some(vr_p_iter)
                            })
                            .collect::<Vec<&Cell>>();
                        if ret1 && ret2.len() != 0 {
                            Some(InferenceResult {
                                inference: self,
                                condition: vp
                                    .iter()
                                    .map(|&p| TheCellAndTheValue {
                                        the_cell: p,
                                        the_value: vec![v],
                                    })
                                    .collect::<Vec<TheCellAndTheValue>>(),
                                conclusion_set_value: None,
                                conclusion_remove_drafts: Some(
                                    ret2.iter()
                                        .map(|&p| TheCellAndTheValue {
                                            the_cell: p,
                                            the_value: vec![v],
                                        })
                                        .collect::<Vec<TheCellAndTheValue>>(),
                                ),
                            })
                        } else {
                            None
                        }
                    }
                })
        })
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r: String = "".to_string();
        inference_result.condition.iter().for_each(|cv| {
            r.push_str(&format!("{:?} ", cv.the_cell.rc));
        });
        r.push_str(&format!(
            "的所有可能 {:?} 在 C{:?} 内都只在 G{:?} 中，推导出：",
            inference_result.condition[0].the_value,
            inference_result.condition[0].the_cell.rc.c + 1,
            inference_result.condition[0].the_cell.gn.g + 1
        ));

        if inference_result.conclusion_remove_drafts.is_some() {
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });
            r.push_str(&format!(
                "均不能填写 {:?} 。",
                inference_result.condition[0].the_value
            ));
        }

        r
    }
}

/// 显性数对排除法（行），在某一行中，存在2/3/4/5数对时，排除该行中其余数对草稿数
/// 定义：X个格子内的候选数字的并集，数量正好是X，称之为【数对】，其中 2<=X<=4
struct RowExplicitPairExclusionInference;
impl Inference for RowExplicitPairExclusionInference {
    fn analyze<'a>(&'a self, field: &'a Field) -> Option<InferenceResult<'a>> {
        // 定义子函数，将一个集合拆分成X和剩余部分的两个集合，且 2<=X<=4
        // 这里生成长度为2/3/4的所有组合的数组索引
        fn generate_combinations(
            len: usize,
            size: usize,
            current: usize,
            path: &mut Vec<usize>,
            all_combinations: &mut Vec<(Vec<usize>, Vec<usize>)>,
        ) {
            // 剪枝：当数对和需要组合的长度相等时，直接返回，没有必要进行判断了
            if path.len() == len {
                return;
            }
            if path.len() == size {
                let mut remaining = Vec::new();
                for set in 0..len {
                    if !path.contains(&set) {
                        remaining.push(set);
                    }
                }
                all_combinations.push((path.clone(), remaining));
                return;
            }
            for i in current..len {
                path.push(i);
                generate_combinations(len, size, i + 1, path, all_combinations);
                path.pop();
            }
        }

        // field.iter_all_drafts_cells_by_rc().for_each(|vr| {
        for vr in field.iter_all_drafts_cells_by_rc() {
            let mut all_combinations = Vec::new();
            for size in 2..=4 {
                let mut paths = Vec::new();
                generate_combinations(vr.len(), size, 0, &mut paths, &mut all_combinations);
            }

            for (combo, rest) in all_combinations {
                let mut union_set = Drafts::default();
                for set in &combo {
                    union_set = union_set.union(vr[*set].drafts);
                }
                let union_set_vec = union_set.to_vec();
                // 检查并集的数量是否等于集合的数量
                if union_set_vec.len() == combo.len() {
                    for tmp in &union_set_vec {
                        let mut tmp_ret = Vec::new();
                        for tmp2 in &rest {
                            if vr[*tmp2].drafts.is_contain(*tmp) {
                                tmp_ret.push(vr[*tmp2]);
                            }
                        }
                        if !tmp_ret.is_empty() {
                            let condition = combo
                                .iter()
                                .map(|tmp4| TheCellAndTheValue {
                                    the_cell: vr[*tmp4],
                                    the_value: union_set_vec.clone(),
                                })
                                .collect();
                            let conclusion = Some(
                                tmp_ret
                                    .iter()
                                    .map(|tmp5| TheCellAndTheValue {
                                        the_cell: tmp5,
                                        the_value: union_set_vec.clone(),
                                    })
                                    .collect(),
                            );
                            field.print();
                            println!(
                                "combo: {:?}, rest: {:?}, union: {:?}, condition: {:?}, conclusion: {:?}",
                                combo,
                                rest,
                                union_set.to_vec(), condition, conclusion
                            );
                            return Some(InferenceResult {
                                inference: self,
                                condition: condition,
                                conclusion_set_value: None,
                                conclusion_remove_drafts: conclusion,
                            });
                        }
                        // 如果tmp_ret是empty，那就不用返回了
                    }
                }
                // println!(
                //     "combo: {:?}, rest: {:?}, union: {:?}",
                //     combo,
                //     rest,
                //     union_set.to_vec()
                // );
            }
        }
        None
    }

    fn write_result(&self, inference_result: &InferenceResult) -> String {
        let mut r: String = "因 ".to_string();
        inference_result.condition.iter().for_each(|cv| {
            r.push_str(&format!("{:?} ", cv.the_cell.rc));
        });
        r.push_str(&"的草稿");
        inference_result.condition.iter().for_each(|cv| {
            r.push_str(&format!("{:?} ", cv.the_cell.rc));
        });
        r.push_str(&"在同一行内形成了数对，推导出：该行内 ");

        if inference_result.conclusion_remove_drafts.is_some() {
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_cell.rc));
                });

            r.push_str(&format!("均不能填写 "));
            inference_result
                .conclusion_remove_drafts
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|cv| {
                    r.push_str(&format!("{:?} ", cv.the_value));
                });
        }

        r
    }
}
