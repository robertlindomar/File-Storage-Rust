-- Defesa em profundidade para isolamento multi-tenant.
-- A tabela `arquivos` passa a exigir um `project_id` na sessao/transacao.

ALTER TABLE arquivos ENABLE ROW LEVEL SECURITY;
ALTER TABLE arquivos FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS arquivos_isolamento_tenant ON arquivos;

CREATE POLICY arquivos_isolamento_tenant
ON arquivos
USING (
    projeto_id = current_setting('app.current_project_id', true)::uuid
)
WITH CHECK (
    projeto_id = current_setting('app.current_project_id', true)::uuid
);
