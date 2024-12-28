pub mod inferences;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests {
    use crate::{
        inferences::InferenceSet,
        types::Field,
        utils::generate_combinations,
    };

    fn sovle(field: &Field) {
        let mut field = field.clone();
        field.print();
        let ifs = InferenceSet::new();
        loop {
            let result = ifs.analyze(&field);
            match result {
                Some(result) => {
                    // println!("{:?}", inference);
                    let newfield = InferenceSet::apply(&field, result);
                    if let Some(conflict) = newfield.find_conflict() {
                        field.print();
                        newfield.print();
                        println!("conflict: {:?}", conflict);
                        break;
                    } else {
                        field = newfield;
                        if field.check_if_finish() {
                            println!("推导完毕!");
                            field.print();
                            break;
                        }
                    }
                }
                None => {
                    println!("无法推导!");
                    field.print();
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
    fn generate_combinations_test() {
        let mut all_combinations = Vec::new();
        for size in 2..=4 {
            let mut paths = Vec::new();
            generate_combinations(9, size, 0, &mut paths, &mut all_combinations);
        }
        println!("{:?}", all_combinations);
    }
}
