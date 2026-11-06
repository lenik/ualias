# Version is injected by packaging/rpm/Makefile via `zfr version`.
# RPM Version cannot contain '-'; use `zfr version -r` (hyphens → '_').
# srcversion is the unsanitized Meson/git version and names the tarball.
%{!?version:%global version 0.0.0}
%{!?srcversion:%global srcversion %{version}}

Name:           ualias
Version:        %{version}
Release:        1%{?dist}
Summary:        look up or append USB device aliases

License:        AGPL-3.0-or-later
URL:            https://github.com/lenik/ualias
Packager:       Lenik <ualias@bodz.net>
Source0:        %{name}-%{srcversion}.tar.xz

BuildRequires:  meson
BuildRequires:  ninja-build
BuildRequires:  pkgconf
BuildRequires:  cargo
BuildRequires:  rustc
BuildRequires:  gettext
BuildRequires:  libgettextpo-dev
BuildRequires:  pkg-config
BuildRequires:  asciidoctor

%description
ualias reads drop-in files under /etc/udev/ualias.d and prints the matching
USB device alias, or appends a new vendor/product/serial pattern to
ualias.d/local when a name is given. Patterns are matched in sorted file
order so specific serials can fall back to a vendor/product wildcard.

%prep
%setup -q -n %{name}-%{srcversion}

%build
meson setup build \
    --prefix=%{_prefix} \
    --bindir=%{_bindir} \
    --datadir=%{_datadir} \
    --mandir=%{_mandir} \
    --sysconfdir=%{_sysconfdir} \
    --localstatedir=%{_localstatedir} \
    --buildtype=plain
meson compile -C build

%install
meson install -C build --destdir=%{buildroot}

%files
%{_bindir}/ualias
%{_datadir}/bash-completion/completions/ualias
%{_mandir}/man1/ualias.1*
%{_datadir}/locale/*/LC_MESSAGES/ualias.mo
%{_mandir}/*/man1/ualias.1*
%{_datadir}/doc/ualias/
%changelog
* Thu Aug 20 2026 Lenik <ualias@bodz.net>
- Align spec with debian/control (Meson, AGPL-3.0-or-later).
- Version comes from `zfr version`, the same method meson.build uses.
