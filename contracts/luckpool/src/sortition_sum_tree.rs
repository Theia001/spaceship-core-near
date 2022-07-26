use crate::*;

type TypeAddress = AccountId;
type TypeKey = String;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
struct SortitionSumTree {
    k: usize,
    stack: Vec<usize>,
    nodes: Vec<u128>,
    ids_to_node_indexes: HashMap<TypeAddress, usize>,
    node_indexes_to_ids: HashMap<usize, TypeAddress>,
}

impl SortitionSumTree {
    pub fn new(k: usize) -> SortitionSumTree {
        SortitionSumTree {
            k,
            stack: Vec::new(),
            nodes: Vec::new(),
            ids_to_node_indexes: HashMap::new(),
            node_indexes_to_ids: HashMap::new(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SortitionSumTrees {
    sortition_sum_trees: HashMap<TypeKey, SortitionSumTree>,
}

impl SortitionSumTrees {
    pub fn new() -> Self {
        SortitionSumTrees {
            sortition_sum_trees: HashMap::new()
        }
    }

    /**
     *  @dev Create a sortition sum tree with a key.
     *  @param _key The key of the new tree.
     *  @param _k The max number of children for each node in the new tree.
     */
    pub fn create_tree(&mut self, key: TypeKey, k: usize) {
        let mut tree: SortitionSumTree = SortitionSumTree::new(k);
        tree.nodes.push(0);
        self.sortition_sum_trees.insert(key, tree);
    }

    /**
     * 这是个内部方法，会被set等方法调用，跟随其他方法一同测试过了
     *  @dev Update the parents of a node until root.
     *  @param _key The key of the tree to update.
     *  @param _tree_index The index of the node to start from.
     *  @param _plus_or_minus Wether to add (true) or substract (false).
     *  @param _value The value to add or substract.
     */
    pub fn update_parents(
        &mut self,
        key: TypeKey,
        tree_index: usize,
        plus_or_minus: bool,
        value: u128,
    ) {
        if let Some(tree) = self.sortition_sum_trees.get_mut(&key) {
            let mut parent_index = tree_index;
            while parent_index != 0 {
                parent_index = (parent_index - 1) / tree.k;
                tree.nodes[parent_index] = if plus_or_minus {
                    tree.nodes[parent_index] + value
                } else {
                    tree.nodes[parent_index] - value
                };
            }
        }
    }

    /**
     *  @dev Set a value of an address in a tree.
     *  @param _key The key of the tree.
     *  @param _value The new value.
     *  @param _id The ID of the value.
     *  `O(log_k(n))` where
     *  `k` is the maximum number of childs per node in the tree,
     *   and `n` is the maximum number of nodes ever appended.
     */
    pub fn set(&mut self, key: TypeKey, value: u128, id: TypeAddress) {
        if let Some(tree) = self.sortition_sum_trees.get_mut(&key) {
            if let Some(_tree_index) = tree.ids_to_node_indexes.get_mut(&id) {
                //node exist
                let tree_index = _tree_index.clone();
                if value == 0 {
                    //new value==0
                    //remove
                    let value = tree.nodes[tree_index];
                    tree.nodes[tree_index.clone()] = 0;
                    tree.stack.push(tree_index);
                    tree.node_indexes_to_ids.remove(&tree_index);
                    tree.ids_to_node_indexes.remove(&id);
                    self.update_parents(key, tree_index, false, value);
                } else if value != tree.nodes[tree_index] {
                    // New value,and!=0
                    // Set.
                    let plus_or_minus = tree.nodes[tree_index] <= value;
                    let plus_or_minus_value: u128 = if plus_or_minus {
                        value - tree.nodes[tree_index.clone()]
                    } else {
                        tree.nodes[tree_index.clone()] - value
                    };
                    tree.nodes[tree_index] = value;
                    self.update_parents(key, tree_index, plus_or_minus, plus_or_minus_value);
                }
            } else {
                if value != 0 {
                    //node not exist
                    let mut _tree_index: usize = 0;
                    if tree.stack.len() == 0 {
                        //no vacant node
                        _tree_index = tree.nodes.len();
                        tree.nodes.push(value);
                        if (_tree_index != 1) && ((_tree_index - 1) % tree.k == 0) {
                            //is the first child node.
                            //move the parent  down
                            let parent_index = _tree_index / tree.k;
                            let parent_id: TypeAddress = tree.node_indexes_to_ids[&parent_index].clone();
                            let new_index = _tree_index + 1;
                            tree.nodes.push(tree.nodes[parent_index]);
                            tree.node_indexes_to_ids.remove(&parent_index);
                            tree.ids_to_node_indexes.insert(parent_id.clone(), new_index);
                            tree.node_indexes_to_ids.insert(new_index, parent_id.clone());
                        }
                    } else {
                        //vacant node
                        _tree_index = tree.stack[tree.stack.len() - 1];
                        tree.stack.pop();
                        tree.nodes[_tree_index] = value;
                    }
                    tree.ids_to_node_indexes.insert(id.clone(), _tree_index);
                    tree.node_indexes_to_ids.insert(_tree_index, id.clone());
                    //update_parents( _key, tree_index, true, _value);
                    self.update_parents(key, _tree_index, true, value);
                }
            }
        }
    }

    /** @dev Gets a specified ID's associated value.
              *  @param _key The key of the tree.
              *  @param _id The ID of the value.
              *  @return value The associated value.
     */
    pub fn stake_of(&self, key: TypeKey, id: TypeAddress) -> u128 {
        if let Some(tree) = self.sortition_sum_trees.get(&key) {
            if let Some(tree_index) = tree.ids_to_node_indexes.get(&id) {
                return tree.nodes[*tree_index];
            }
        }
        return 0;
    }

    /**
     *  @dev Draw an ID from a tree using a number. Note that this function reverts if the sum of all values in the tree is 0.
     *  @param _key The key of the tree.
     *  @param _drawn_number The drawn number.
     *  @return ID The drawn ID.
     *  `O(k * log_k(n))` where
     *  `k` is the maximum number of childs per node in the tree,
     *   and `n` is the maximum number of nodes ever appended.
     */
    pub fn draw(&self, key: TypeKey, drawn_number: u128) -> TypeAddress {
        if let Some(tree) = self.sortition_sum_trees.get(&key) {
            let mut tree_index: usize = 0;
            let mut current_drawn_number = drawn_number % tree.nodes[0];
            while (tree.k * tree_index) + 1 < tree.nodes.len() {
                for i in 1..=tree.k {
                    let node_index = (tree.k * tree_index) + i;
                    let node_value = tree.nodes[node_index];
                    if current_drawn_number >= node_value {
                        current_drawn_number = current_drawn_number - node_value;
                    } else {
                        tree_index = node_index;
                        break;
                    }
                }
            }
            return tree.node_indexes_to_ids[&tree_index].clone();
        }

        return AccountId::new_unchecked("00".to_string());
    }

    /**
     *  @dev Query the leaves of a tree. Note that if `startIndex == 0`, the tree is empty and the root node will be returned.
     *  @param key The key of the tree to get the leaves from.
     *  @param cursor The pagination cursor.
     *  @param count The number of items to return.
     *  @return startIndex The index at which leaves start.
     *  @return values The values of the returned leaves.
     *  @return hasMore Whether there are more for pagination.
     *  `O(n)` where
     *  `n` is the maximum number of nodes ever appended.
     */
    pub fn query_leaves(
        &self,
        key: TypeKey,
        cursor: usize,
        count: usize,
    ) -> (usize, Vec<u128>, bool) {
        let mut start_index: usize = 0;
        let mut values: Vec<u128> = Vec::new();
        let mut has_more: bool = false;
        if let Some(tree) = self.sortition_sum_trees.get(&key) {
            for i in 1..=tree.nodes.len() {
                if (tree.k) + 1 >= tree.nodes.len() {
                    start_index = i;
                    break;
                }
            }
            let loop_start_index = start_index + cursor;
            for j in loop_start_index..tree.nodes.len() {
                if values.len() < count {
                    values.push(tree.nodes[j]);
                } else {
                    has_more = true;
                    break;
                }
            }
        }
        return (start_index, values, has_more);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_set_test_2_node() {
        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 2); // 节点数
        trees.set(key.clone(), 25, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 25, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 25, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 25, AccountId::new_unchecked("04".to_string()));

        let tree = trees.sortition_sum_trees.get(&key.clone());

        assert_eq!(tree.unwrap().nodes[0], 100);
        assert_eq!(tree.unwrap().nodes[1], 50);
        assert_eq!(tree.unwrap().nodes[2], 50);
        assert_eq!(tree.unwrap().nodes[3], 25);
        assert_eq!(tree.unwrap().nodes[4], 25);
        assert_eq!(tree.unwrap().nodes[5], 25);
        assert_eq!(tree.unwrap().nodes[6], 25);
    }

    #[test]
    fn create_and_set_test_3_node() {
        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 3); // 节点数
        trees.set(key.clone(), 20, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("04".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("05".to_string()));

        let tree = trees.sortition_sum_trees.get(&key.clone());

        assert_eq!(tree.unwrap().nodes[0], 100);
        assert_eq!(tree.unwrap().nodes[1], 60);
        assert_eq!(tree.unwrap().nodes[2], 20);
        assert_eq!(tree.unwrap().nodes[3], 20);
        assert_eq!(tree.unwrap().nodes[4], 20);
        assert_eq!(tree.unwrap().nodes[5], 20);
        assert_eq!(tree.unwrap().nodes[6], 20);

        // 不同 value
        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 3); // 节点数
        trees.set(key.clone(), 10, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 25, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 15, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("04".to_string()));
        trees.set(key.clone(), 45, AccountId::new_unchecked("05".to_string()));

        let tree = trees.sortition_sum_trees.get(&key.clone());

        assert_eq!(tree.unwrap().nodes[0], 100);
        assert_eq!(tree.unwrap().nodes[1], 60);
        assert_eq!(tree.unwrap().nodes[2], 25);
        assert_eq!(tree.unwrap().nodes[3], 15);
        assert_eq!(tree.unwrap().nodes[4], 5);
        assert_eq!(tree.unwrap().nodes[5], 10);
        assert_eq!(tree.unwrap().nodes[6], 45);
    }

    #[test]
    fn create_and_set_test_5_node() {
        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 5); // 节点数
        trees.set(key.clone(), 5, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("04".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("05".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("06".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("07".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("08".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("09".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("10".to_string()));

        trees.set(key.clone(), 5, AccountId::new_unchecked("11".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("12".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("13".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("14".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("15".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("16".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("17".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("18".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("19".to_string()));
        trees.set(key.clone(), 5, AccountId::new_unchecked("20".to_string()));

        let tree = trees.sortition_sum_trees.get(&key.clone());

        assert_eq!(tree.unwrap().nodes[0], 100);
        assert_eq!(tree.unwrap().nodes[1], 25);
        assert_eq!(tree.unwrap().nodes[2], 25);
        assert_eq!(tree.unwrap().nodes[3], 25);
        assert_eq!(tree.unwrap().nodes[4], 20);
        assert_eq!(tree.unwrap().nodes[5], 5);
        assert_eq!(tree.unwrap().nodes[6], 5);
        assert_eq!(tree.unwrap().nodes[7], 5);
        assert_eq!(tree.unwrap().nodes[8], 5);
        assert_eq!(tree.unwrap().nodes[9], 5);
        assert_eq!(tree.unwrap().nodes[10], 5);
        assert_eq!(tree.unwrap().nodes[11], 5);
        assert_eq!(tree.unwrap().nodes[12], 5);
        assert_eq!(tree.unwrap().nodes[13], 5);
        assert_eq!(tree.unwrap().nodes[14], 5);
        assert_eq!(tree.unwrap().nodes[15], 5);
        assert_eq!(tree.unwrap().nodes[16], 5);
        assert_eq!(tree.unwrap().nodes[17], 5);
        assert_eq!(tree.unwrap().nodes[18], 5);
        assert_eq!(tree.unwrap().nodes[19], 5);
        assert_eq!(tree.unwrap().nodes[20], 5);
        assert_eq!(tree.unwrap().nodes[21], 5);
        assert_eq!(tree.unwrap().nodes[22], 5);
        assert_eq!(tree.unwrap().nodes[23], 5);
        assert_eq!(tree.unwrap().nodes[24], 5);
    }

    // 后面测试是建立在前面 x_node 测试成功情况下，部分代码与前面相同
    #[test]
    fn stake_of_test() {
        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 5); // 节点数
        trees.set(key.clone(), 20, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("04".to_string()));
        trees.set(key.clone(), 20, AccountId::new_unchecked("05".to_string()));

        // 查找存在的值 01
        let value = trees.stake_of(key.clone(), AccountId::new_unchecked("01".to_string()));
        assert_eq!(value, 20);

        let value = trees.stake_of(key.clone(), AccountId::new_unchecked("02".to_string()));
        assert_eq!(value, 20);

        let value = trees.stake_of(key.clone(), AccountId::new_unchecked("03".to_string()));
        assert_eq!(value, 20);

        // 查找一个不存在的值 06
        let value = trees.stake_of(key.clone(), AccountId::new_unchecked("06".to_string()));
        assert_eq!(value, 0)
    }

    #[test]
    fn draw_test() {
        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 5); // 节点数
        trees.set(key.clone(), 1, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 22, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 31, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 40, AccountId::new_unchecked("04".to_string()));
        trees.set(key.clone(), 4, AccountId::new_unchecked("05".to_string()));
        trees.set(key.clone(), 2, AccountId::new_unchecked("06".to_string()));

        let draw_result = trees.draw(key.clone(), 41);
        assert_eq!(draw_result, AccountId::new_unchecked("03".to_string()));

        let draw_result = trees.draw(key.clone(), 55);
        assert_eq!(draw_result, AccountId::new_unchecked("03".to_string()));

        let draw_result = trees.draw(key.clone(), 99);
        assert_eq!(draw_result, AccountId::new_unchecked("05".to_string()));


        let key = "firstTree".to_string();
        let mut trees = SortitionSumTrees::new();
        trees.create_tree(key.clone(), 5); // 节点数
        trees.set(key.clone(), 1, AccountId::new_unchecked("01".to_string()));
        trees.set(key.clone(), 22, AccountId::new_unchecked("02".to_string()));
        trees.set(key.clone(), 31, AccountId::new_unchecked("03".to_string()));
        trees.set(key.clone(), 40, AccountId::new_unchecked("04".to_string()));
        trees.set(key.clone(), 6, AccountId::new_unchecked("05".to_string()));

        let draw_result = trees.draw(key.clone(), 41);
        assert_eq!(draw_result, AccountId::new_unchecked("03".to_string()));

        let draw_result = trees.draw(key.clone(), 55);
        assert_eq!(draw_result, AccountId::new_unchecked("04".to_string())); // 注意这里和前一个不同

        let draw_result = trees.draw(key.clone(), 99);
        assert_eq!(draw_result, AccountId::new_unchecked("05".to_string()));
    }
}
