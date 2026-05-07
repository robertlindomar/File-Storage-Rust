-- Leitura cross-tenant para administradores: so quando a sessao define app.admin_global_read.
-- Combinacao OR com a politica existente de tenant (0003_arquivos_rls.sql).

CREATE POLICY arquivos_leitura_admin_global ON arquivos
FOR SELECT
USING (current_setting('app.admin_global_read', true) = 'true');
