pub struct DetectResult<T> {
    pub create: Vec<T>,
    pub delete: Vec<T>
}

pub fn detect_differences_in_vecs<T: PartialEq + Eq + std::hash::Hash + Clone>(v1: Vec<T>, v2: Vec<T>) -> Option<DetectResult<T>> {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    let old: HashSet<&T, std::hash::RandomState> = HashSet::from_iter(v1.iter());
    let new: HashSet<&T, std::hash::RandomState> = HashSet::from_iter(v2.iter());

    if old == new {
        return None;
    }
    
    let mut merged: HashSet<&T, std::hash::RandomState> = old.clone();
    merged.extend(&new);
    let merged = merged;

    let mut to_add: HashSet<&T, std::hash::RandomState> = merged.clone();
    let mut to_delete: HashSet<&T, std::hash::RandomState> = HashSet::new();

    for item in merged {
        let old_contains = old.contains(item);
        let new_contains = new.contains(item);
        
        // not found in new
        if old_contains && !new_contains {
            // move to to_delete 
            to_add.remove(item);
            to_delete.insert(item);

            continue;
        }

        // found in both
        if old_contains && new_contains {
            // delete from to_add
            to_add.remove(item);
        }

        // else do nothing
    }

    let create: Vec<T> = to_add
        .into_iter()
        .cloned()
        .collect();
    
    let delete: Vec<T> = to_delete
        .into_iter()
        .cloned()
        .collect();

    return Some(DetectResult { create, delete });
}