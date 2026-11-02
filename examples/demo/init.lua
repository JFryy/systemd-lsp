-- Minimal Neovim config used to record examples/demo.gif (see demo.tape).
-- Expects SYSTEMD_LSP_BIN to point at a built systemd-lsp binary.

vim.o.termguicolors = true
vim.o.number = true
vim.o.signcolumn = "yes"
vim.o.swapfile = false
vim.o.showmode = false
vim.o.shortmess = vim.o.shortmess .. "Ic"
vim.o.completeopt = "menuone,noinsert"
vim.o.pumheight = 8
vim.o.winborder = "rounded"
vim.o.updatetime = 200

vim.filetype.add({
  extension = {
    service = "systemd",
    socket = "systemd",
    timer = "systemd",
    mount = "systemd",
    automount = "systemd",
    swap = "systemd",
    target = "systemd",
    path = "systemd",
    slice = "systemd",
    scope = "systemd",
    device = "systemd",
    container = "podman",
    volume = "podman",
    network = "podman",
    kube = "podman",
    pod = "podman",
    build = "podman",
    image = "podman",
  },
})

vim.lsp.config("systemd_lsp", {
  cmd = { vim.env.SYSTEMD_LSP_BIN or "systemd-lsp" },
  filetypes = { "systemd", "podman" },
})
vim.lsp.enable("systemd_lsp")

vim.diagnostic.config({
  virtual_text = { prefix = "●", spacing = 2 },
  severity_sort = true,
  update_in_insert = true,
})

vim.api.nvim_create_autocmd("LspAttach", {
  callback = function(ev)
    local client = assert(vim.lsp.get_client_by_id(ev.data.client_id))
    -- Open the completion menu while typing keys, not only on "[" and "=".
    local triggers = { "[", "=" }
    for c in ("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"):gmatch(".") do
      table.insert(triggers, c)
    end
    client.server_capabilities.completionProvider.triggerCharacters = triggers
    vim.lsp.completion.enable(true, client.id, ev.buf, { autotrigger = true })
  end,
})

vim.keymap.set("i", "<CR>", function()
  return vim.fn.pumvisible() == 1 and "<C-y>" or "<CR>"
end, { expr = true })
