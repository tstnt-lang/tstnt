# TSTNT Neovim syntax highlighting

## Install

```bash
mkdir -p ~/.config/nvim/syntax
cp tstnt.vim ~/.config/nvim/syntax/

# Add to ~/.config/nvim/init.vim:
au BufRead,BufNewFile *.tstnt set filetype=tstnt
```

## Vim (not Neovim)

```bash
mkdir -p ~/.vim/syntax
cp tstnt.vim ~/.vim/syntax/
# Add to ~/.vimrc:
au BufRead,BufNewFile *.tstnt set filetype=tstnt
```
