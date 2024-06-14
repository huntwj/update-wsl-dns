alias symlink := sym-link

desired_link_target := "/etc/resolv.conf.autodetect"
current_link_target := ```readlink -f "/etc/resolv.conf"```

update:
  @cargo --quiet run | sudo tee {{ desired_link_target }} > /dev/null
  @echo Success! {{ desired_link_target }} written.
  @{{ if current_link_target == desired_link_target { "just display-link" } else { "echo Note: /etc/resolv.conf is not linked correctly." } }}

sym-link:
  cd /etc && sudo ln --symbolic --force resolv.conf.autodetect resolv.conf
  @echo Success!
  @just display-link

display-link:
  @echo
  @echo And /etc/resolv.conf is correctly linked:
  @ls -o /etc/resolv.conf
  