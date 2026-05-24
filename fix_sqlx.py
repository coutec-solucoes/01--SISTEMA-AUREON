import os
import re

files = [
    r'services\aureon-api-local\src\routes\fiscal\configuracoes.rs',
    r'services\aureon-api-local\src\routes\fiscal\dicionarios.rs',
    r'services\aureon-api-local\src\routes\fiscal\regras.rs',
    r'services\aureon-api-local\src\routes\fiscal\versoes.rs',
]

for fpath in files:
    with open(fpath, 'r', encoding='utf-8') as f:
        content = f.read()

    if 'use sqlx::Row;' not in content:
        content = content.replace('use crate::app::AppState;', 'use sqlx::Row;\nuse crate::app::AppState;')

    # Fix queries
    content = content.replace('sqlx::query!(', 'sqlx::query(')
    
    # Fix row access
    content = re.sub(r'row\.([a-zA-Z0-9_]+)\.to_string\(\)', r'row.get::<uuid::Uuid, _>("\1").to_string()', content)
    content = re.sub(r'row\.([a-zA-Z0-9_]+)\.map\(\|u\| u\.to_string\(\)\)', r'row.get::<Option<uuid::Uuid>, _>("\1").map(|u| u.to_string())', content)
    content = re.sub(r'row\.([a-zA-Z0-9_]+)\.map\(\|d\| d\.to_string\(\)\)', r'row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("\1").map(|d| d.to_string())', content)
    
    # For remaining simple row.fields that are NOT already row.get(
    # negative lookbehind for . and negative lookahead for (
    content = re.sub(r'(?<!\.)row\.([a-zA-Z0-9_]+)(?!\()', r'row.get("\1")', content)

    with open(fpath, 'w', encoding='utf-8') as f:
        f.write(content)

print('Done')
