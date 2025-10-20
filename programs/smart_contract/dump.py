from pathlib import Path

# Dump the content of all .rs files in the current directory and its subdirectories into tmp.txt
output_file = Path("tmp.txt")

with output_file.open("w", encoding="utf-8") as out:
    for file in sorted(Path(".").rglob("*.rs")):
        out.write(f"\n----- FILE: {file} -----\n\n")
        try:
            out.write(file.read_text(encoding="utf-8"))
        except UnicodeDecodeError:
            out.write("[Erreur de lecture: fichier non UTF-8]\n")

print(f"✅ Contenu de tous les fichiers .rs écrit dans {output_file.resolve()}")
