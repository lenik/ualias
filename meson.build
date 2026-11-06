project(
    'ualias',
    'rust', # Required by i18n.gettext; no C is compiled in this project.
    version: run_command(
        'sh',
        '-c', '''
        v=$(
            command -v zfr >/dev/null 2>&1 && zfr version 2>/dev/null || true
        )
        if [ -z "$v" ] && [ -f VERSION ]; then
            v=$(head -n1 VERSION | tr -d '\r')
            v=${v#v}
        fi
        if [ -z "$v" ]; then
            v="0.0.0" # FIXED TO 0.0.0, DO NOT MODIFY
        fi
        printf '%s' "$v"
        ''',
        check: false,
    ).stdout().strip(),
    license: 'AGPL-3.0-or-later',
    default_options: [
        'warning_level=0',
    ],
    meson_version: '>=0.60.0',
)

project_author = 'Lenik'
project_email = 'ualias@bodz.net'
project_year = 2026

fs = import('fs')

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
localedir = prefix / get_option('localedir')
mandir = prefix / get_option('mandir')
pkgdocdir = datadir / 'doc' / meson.project_name()

# Rust binary (release) built with Cargo, installed as ualias
cargo_build = find_program('build-aux/cargo-build.sh', dirs: meson.project_source_root(), required: true)
asciidoctor = find_program('asciidoctor', required: true)

ualias = custom_target(
    'ualias',
    output: 'ualias',
    input: 'Cargo.lock',
    depend_files: [
        'Cargo.toml',
        'src/lib.rs',
        'src/main.rs',
    ],
    command: [cargo_build, '@OUTPUT@', meson.project_source_root(), meson.current_build_dir()],
    build_by_default: true,
    install: true,
    install_dir: bindir,
)

bash_files = [
    'ualias.bash',
]

foreach file : bash_files
    name = fs.stem(file)
    install_data(
        file,
        install_dir: datadir / 'bash-completion' / 'completions',
        rename: name,
    )
endforeach

install_data(
    [
        'LICENSE',
        'README.md',
        'README-zh_CN.md',
    ],
    install_dir: pkgdocdir,
)

custom_target(
    'ualias-man',
    input: 'man/ualias.adoc',
    output: 'ualias.1',
    command: [
        asciidoctor,
        '-b', 'manpage',
        '-a', 'project-version=' + meson.project_version(),
        '-a', 'project-year=@0@'.format(project_year),
        '-a', 'project-author=' + project_author,
        '-a', 'project-email=' + project_email,
        '-o', '@OUTPUT@',
        '@INPUT@',
    ],
    build_by_default: true,
    install: true,
    install_dir: mandir / 'man1',
)

# Whole-document translated man pages → $mandir/<locale>/man1/
man_locales = ['ar', 'de', 'es', 'fr', 'it', 'ja', 'ko', 'zh_CN']
foreach loc : man_locales
    custom_target(
        'ualias-man-' + loc,
        input: 'man' / loc / 'ualias.adoc',
        output: loc + '/man1/ualias.1',
        command: [
            asciidoctor,
            '-b', 'manpage',
            '-a', 'project-version=' + meson.project_version(),
            '-a', 'project-year=@0@'.format(project_year),
            '-a', 'project-author=' + project_author,
            '-a', 'project-email=' + project_email,
            '-o', '@OUTPUT@',
            '@INPUT@',
        ],
        build_by_default: true,
        install: true,
        install_dir: mandir,
    )
endforeach

subdir('po')

# look task: list what installed files looks like
# ninja install <tmpdir> && tree <tmpdir> && rm -fr <tmpdir>
run_target(
    'look',
    command: [
        find_program('zfr'),
        'build',
        '--look',
        '-C', meson.project_source_root(),
        '-b', meson.project_build_root(),
    ],
)

# Sync gettext catalogs (xgettext / msgmerge) via zfr.
run_target(
    'posync',
    command: [
        'bash',
        meson.project_source_root() / 'scripts' / 'posync.sh',
        meson.project_source_root(),
        meson.project_build_root(),
    ],
)

# Fast local debugging against the active installation prefix:
# symlink installed paths to project/build outputs.
run_target(
    'install-symlinks',
    command: [
        'bash',
        meson.project_source_root() / 'scripts' / 'install-symlinks.sh',
        meson.project_source_root(),
        meson.project_build_root(),
        get_option('prefix'),
        bindir,
        datadir,
        localedir,
        mandir,
    ],
)

run_target(
    'uninstall-symlinks',
    command: [
        'bash',
        meson.project_source_root() / 'scripts' / 'uninstall-symlinks.sh',
        meson.project_source_root(),
        meson.project_build_root(),
        get_option('prefix'),
        bindir,
        datadir,
        localedir,
        mandir,
    ],
)

env_test = environment()
env_test.set('RUSTC_WRAPPER', '')

test(
    'cargo-test',
    find_program('cargo', required: true),
    args: [
        'test',
        '--locked',
        '--manifest-path',
        meson.project_source_root() / 'Cargo.toml',
        '--target-dir',
        meson.current_build_dir() / 'cargo-test-target',
    ],
    workdir: meson.project_source_root(),
    env: env_test,
    timeout: 300,
)
