from pathlib import Path
import shutil

root = Path('/home/g22/sem-rvm002-prototype')
src = root / 'crates/sm-vm/src/semcode_vm.rs'
text = src.read_text()
assert text.count('Map(Vec<(MapKey, Value)>)') == 1
text = text.replace('Map(Vec<(MapKey, Value)>)', 'Map(MapValue)')
text = '#[path = "map_value_rvm002.rs"]\nmod map_value_rvm002;\nuse map_value_rvm002::MapValue;\n' + text
text = text.replace('Value::Map(Vec::new())', 'Value::Map(MapValue::default())')
text = text.replace('pairs.iter().any(|(k, _)| k == &key)', 'pairs.contains(&key)')
text = text.replace('pairs.iter().find(|(k, _)| k == &key)', 'pairs.get(&key)')
text = text.replace('Some((_, v)) => v.clone()', 'Some(v) => v.clone()')
text = text.replace('map_updated_pairs(pairs, key, val)', 'pairs.updated(key, val)')
src.write_text(text)
shutil.copyfile('/mnt/c/Users/said3/Documents/Codex/SEM-RVM-002/map_value_vm.rs', root/'crates/sm-vm/src/map_value_rvm002.rs')
