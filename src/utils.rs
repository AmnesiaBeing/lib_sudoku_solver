// 这里放置一些常用的工具类

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
