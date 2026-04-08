# File Storage Service — uso (multi-tenant)

Serviço HTTP em Rust (Axum) que guarda ficheiros em disco e metadados no PostgreSQL. Cada **projeto** tem a sua própria **API key**; rotas administrativas usam **`API_KEY_ADMIN`**.

## Pré-requisitos

- **Rust** (toolchain estável) e **Cargo**
- **PostgreSQL** acessível por `DATABASE_URL`
- Base de dados criada, por exemplo:

```sql
CREATE DATABASE file_storage;
```

## Variáveis de ambiente

| Variável | Descrição |
|----------|-----------|
| `DATABASE_URL` | URL PostgreSQL (ex.: `postgresql://user:pass@localhost:5432/file_storage`) |
| `DIRETORIO_ARMAZENAMENTO` | Pasta raiz no disco; ficheiros ficam em `{DIRETORIO_ARMAZENAMENTO}/{projeto_id}/{arquivo_id}` |
| `PORTA` | Porta HTTP (default `3000`) |
| `API_KEY_ADMIN` | Bearer para `POST/GET/DELETE /api/v1/admin/projetos` (default de desenvolvimento no código: trocar em produção) |
| `TAMANHO_MAXIMO_ARQUIVO_BYTES` | Limite por upload em bytes (default 100 MiB) |
| `BASE_URL` | Opcional; URL pública usada nas respostas de upload (`id` + `url`). Se vazio, usa `http://localhost:{PORTA}` |

Copie `.env.example` para `.env` e ajuste.

## Migrações e arranque

Na raiz do projeto:

```bash
sqlx database create   # opcional, se ainda não existir a DB
sqlx migrate run
cargo run
```

O binário carrega `.env` automaticamente em desenvolvimento (via `dotenvy`).

## Autenticação

- **Rotas de projeto** (`/api/v1/arquivos`, `/api/v1/imagens`): header `Authorization: Bearer <api_key_do_projeto>`.
- **Rotas admin** (`/api/v1/admin/projetos`): `Authorization: Bearer <API_KEY_ADMIN>`.

## Exemplos com curl

Substitua `ADMIN_KEY`, `BASE` e ficheiros conforme o seu ambiente.

### 1. Criar um projeto (admin)

```bash
curl -sS -X POST "$BASE/api/v1/admin/projetos" \
  -H "Authorization: Bearer $ADMIN_KEY" \
  -H "Content-Type: application/json" \
  -d '{"nome":"Meu projeto"}'
```

Resposta (201): JSON com `id`, `nome` e `api_key`. Guarde `api_key` como `PROJETO_KEY`.

### 2. Enviar um ficheiro (multipart, campo `arquivo`)

```bash
curl -sS -X POST "$BASE/api/v1/arquivos" \
  -H "Authorization: Bearer $PROJETO_KEY" \
  -F "arquivo=@/caminho/para/ficheiro.pdf"
```

### 3. Listar e descarregar

```bash
curl -sS "$BASE/api/v1/arquivos" -H "Authorization: Bearer $PROJETO_KEY"
curl -sS -OJ "$BASE/api/v1/arquivos/$ARQUIVO_ID" -H "Authorization: Bearer $PROJETO_KEY"
```

### 4. Imagem (validação de tipo)

```bash
curl -sS -X POST "$BASE/api/v1/imagens" \
  -H "Authorization: Bearer $PROJETO_KEY" \
  -F "arquivo=@/caminho/para/foto.png"
```

### 5. Apagar projeto (admin; remove ficheiros em cascade na BD e a pasta do projeto no disco)

Resposta **200** com JSON do projeto apagado (`id`, `nome`, `api_key`, `criado_em`).

```bash
curl -sS -X DELETE "$BASE/api/v1/admin/projetos/$PROJETO_ID" \
  -H "Authorization: Bearer $ADMIN_KEY"
```

## Saúde (sem autenticação)

- `GET /health` — liveness
- `GET /ready` — readiness (verifica PostgreSQL)

## Coleção Postman

Ver `postman/FileStorege.postman_collection.json`: variáveis `baseUrl`, `tokenAdmin` (admin), `token` (API key do projeto; o script do POST “criar projeto” pode preencher a partir da resposta).
