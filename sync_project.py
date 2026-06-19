import json
import os
import re
import subprocess
import time

def run_cmd(cmd):
    print("Running:", cmd)
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print("Error:", result.stderr)
    return result.stdout.strip()

def main():
    log_file = r"C:\Users\said3\.gemini\antigravity\brain\28196170-5b86-41f1-8a24-f902fc30503f\.system_generated\tasks\task-2338.log"
    prs = {}
    with open(log_file, "r", encoding="utf-8") as f:
        lines = f.readlines()
        
    current_pr = None
    for line in lines:
        line = line.strip()
        m = re.match(r"=== PR #(\d+) ===", line)
        if m:
            current_pr = int(m.group(1))
            prs[current_pr] = {"number": current_pr, "json": None, "sha": None}
            continue
        
        if current_pr and line.startswith("{") and line.endswith("}"):
            try:
                prs[current_pr]["json"] = json.loads(line)
            except:
                pass
            continue
            
        if current_pr and line and not line.startswith("{") and not line.startswith("merge"):
            if "changed_files" not in prs[current_pr]:
                prs[current_pr]["changed_files"] = []
            prs[current_pr]["changed_files"].append(line)

    project_items_path = "project_items.json"
    project_items = []
    if os.path.exists(project_items_path):
        try:
            with open(project_items_path, "r", encoding="utf-16") as f:
                for line in f:
                    if line.startswith("{") and "items" in line and "content" not in line:
                        continue
                    try:
                        data = json.loads(line)
                        if "items" in data and isinstance(data["items"], list):
                            project_items = data["items"]
                            break
                        if "items" in data:
                            project_items = data["items"]
                    except:
                        pass
        except:
            pass

    if not project_items:
        try:
            with open(project_items_path, "r", encoding="utf-16") as f:
                data = json.load(f)
                if "items" in data:
                    project_items = data["items"]
        except:
            pass

    proj_by_pr = {}
    for item in project_items:
        content = item.get("content", {})
        if "number" in content and content.get("type") == "PullRequest":
            num = content["number"]
            if num not in proj_by_pr:
                proj_by_pr[num] = []
            proj_by_pr[num].append(item)

    ui_keywords = ["prom-ui", "apps/workbench", "docs/workbench", "docs/roadmap/post_ui", "SEMANTIC_UI_DNA.md", "renderer", "projection", "layout", "tree", "ast", "ir", "capability", "workbench", "studio"]
    
    # Missing / Wrong / Duplicated processing
    for num in range(913, 1104):
        pr = prs.get(num)
        if not pr or not pr["json"]: continue
            
        data = pr["json"]
        title = data.get("title", "")
        state = data.get("state", "UNKNOWN")
        files = pr.get("changed_files", [])
        
        is_ui = False
        title_ui = "ui" in title.lower() or "renderer" in title.lower() or "layout" in title.lower()
        for f in files:
            for kw in ui_keywords:
                if kw in f.lower():
                    is_ui = True
                    break
            if is_ui: break
            
        if title_ui: is_ui = True
        if not is_ui: continue

        has_src = any("src/" in f for f in files)
        has_tests = any("tests/" in f for f in files)
        has_docs = any("docs/" in f for f in files)
        
        pr_type = "mixed"
        if has_src and not has_tests and not has_docs: pr_type = "source"
        elif has_tests and not has_src and not has_docs: pr_type = "test"
        elif has_docs and not has_src and not has_tests: pr_type = "docs"
        elif has_src: pr_type = "source"
        
        if title.startswith("docs("): pr_type = "docs"
        if title.startswith("test("): pr_type = "test"
        if title.startswith("feat("): pr_type = "source"
        
        p_items = proj_by_pr.get(num, [])
        expected_status = "Done" if state == "MERGED" else ("In Progress" if state == "OPEN" else "Todo")
        
        if len(p_items) == 0:
            print(f"Adding PR #{num} to project...")
            out = run_cmd(f"gh project item-add 2 --owner skulmakov-oss --url https://github.com/skulmakov-oss/Semantic/pull/{num} --format json")
            try:
                new_item = json.loads(out)
                p_items = [new_item]
                # small delay to prevent rate limits
                time.sleep(1)
            except Exception as e:
                print("Failed to parse gh output for add item", e)
                
        elif len(p_items) > 1:
            print(f"Removing duplicate items for PR #{num}...")
            # keep the first one, delete rest
            for dup_item in p_items[1:]:
                item_id = dup_item["id"]
                run_cmd(f"gh project item-delete 2 --owner skulmakov-oss --id {item_id}")
            p_items = [p_items[0]]

        # Status & metadata update
        if p_items:
            item = p_items[0]
            item_id = item["id"]
            p_status = item.get("status", "")
            
            # Update status if needed
            if p_status != expected_status:
                print(f"Updating status for PR #{num} to {expected_status}...")
                opt_id = "98236657" if expected_status == "Done" else "47fc9ee4"
                run_cmd(f"gh project item-edit --id {item_id} --project-id PVT_kwHOD49aK84BRzo5 --field-id PVTSSF_lAHOD49aK84BRzo5zg_hoYM --single-select-option-id {opt_id}")
                
            # Default missing metadata
            # Field IDs:
            # Track = PVTSSF_lAHOD49aK84BRzo5zhVFWlw (POST-UI: bb0800a1)
            # Wave = PVTSSF_lAHOD49aK84BRzo5zhVFWl0 (R12: 6b671f04)
            # Type = PVTSSF_lAHOD49aK84BRzo5zhVFWms (Code: f6d9da89, Docs: db482e62, Test: 8964b6c0, Roadmap: 08dbae30, Audit: 34fcd9d8, Closeout: 9213e5a2)
            # Risk = PVTSSF_lAHOD49aK84BRzo5zhVFWnk (Medium: 3f3f9ae4)
            # Boundary = PVTSSF_lAHOD49aK84BRzo5zhVFWno (Semantic UI: 214e0991, Renderer: fb26f0e3)
            # Gate = PVTSSF_lAHOD49aK84BRzo5zhVFWns (PRReady: 5328edab, Docs-only: d79c49e6, FullPreflight: 98dbff78, Release Artifact: 9ae54490, Planning-only: 9c98a8c9)

            track_id = "PVTSSF_lAHOD49aK84BRzo5zhVFWlw"
            wave_id = "PVTSSF_lAHOD49aK84BRzo5zhVFWl0"
            type_id = "PVTSSF_lAHOD49aK84BRzo5zhVFWms"
            risk_id = "PVTSSF_lAHOD49aK84BRzo5zhVFWnk"
            boundary_id = "PVTSSF_lAHOD49aK84BRzo5zhVFWno"
            gate_id = "PVTSSF_lAHOD49aK84BRzo5zhVFWns"
            
            updates = []
            
            if not item.get("track"): updates.append(f"--field-id {track_id} --single-select-option-id bb0800a1") # POST-UI
            if not item.get("wave"): updates.append(f"--field-id {wave_id} --single-select-option-id 6b671f04") # R12
            if not item.get("risk"): updates.append(f"--field-id {risk_id} --single-select-option-id 3f3f9ae4") # Medium
            
            if not item.get("boundary"):
                if "renderer" in title.lower() or "layout" in title.lower():
                    updates.append(f"--field-id {boundary_id} --single-select-option-id fb26f0e3") # Renderer
                else:
                    updates.append(f"--field-id {boundary_id} --single-select-option-id 214e0991") # Semantic UI
                    
            if not item.get("type"):
                t_val = "db482e62" # Docs
                if "audit" in title.lower(): t_val = "34fcd9d8" # Audit
                elif "closeout" in title.lower(): t_val = "9213e5a2" # Closeout
                elif "select next" in title.lower() or "roadmap" in title.lower(): t_val = "08dbae30" # Roadmap
                elif pr_type == "source": t_val = "f6d9da89" # Code
                elif pr_type == "test": t_val = "8964b6c0" # Test
                updates.append(f"--field-id {type_id} --single-select-option-id {t_val}")
                
            if not item.get("gate"):
                g_val = "98dbff78" # FullPreflight
                if pr_type == "docs":
                    if "audit" in title.lower(): g_val = "98dbff78"
                    elif "closeout" in title.lower(): g_val = "9ae54490"
                    elif "select next" in title.lower() or "roadmap" in title.lower(): g_val = "9c98a8c9"
                    else: g_val = "d79c49e6" # Docs-only
                elif pr_type == "source": g_val = "5328edab" # PRReady
                updates.append(f"--field-id {gate_id} --single-select-option-id {g_val}")

            for update_arg in updates:
                print(f"Updating metadata for PR #{num}: {update_arg}")
                run_cmd(f"gh project item-edit --id {item_id} --project-id PVT_kwHOD49aK84BRzo5 {update_arg}")
                
    print("Project synchronization completed.")

if __name__ == "__main__":
    main()
