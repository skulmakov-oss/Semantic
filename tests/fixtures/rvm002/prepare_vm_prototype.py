from pathlib import Path
import shutil
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
assert (root / '.git').is_dir(), 'use a dedicated disposable clone'
head = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
assert head == '8db588bea6395839eb543aee453c2b2657e856d5', 'wrong prototype base'
assert not subprocess.check_output(['git', '-C', str(root), 'status', '--porcelain', '--untracked-files=no'], text=True), 'tracked tree must be pristine'
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
shutil.copyfile(Path(__file__).with_name('map_value_vm.rs'), root/'crates/sm-vm/src/map_value_rvm002.rs')
