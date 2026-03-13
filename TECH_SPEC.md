ТЕХНИЧЕСКОЕ ЗАДАНИЕ
Rust-обёртка над official Google JPEGli

1. Цель

Разработать Rust-библиотеку-обёртку над official Google JPEGli со следующими свойствами:

- использование official Google JPEGli как upstream-реализации;
- permissive-лицензионный стек со стороны JPEGli upstream;
- автоматическая сборка C/C++-библиотеки из исходников;
- безопасный Rust API для encode-first сценариев;
- возможность дальнейшего расширения до decode API;
- отсутствие зависимости от AGPL-проектов.

2. Результат работ

По итогам работ должен быть предоставлен репозиторий/workspace, содержащий:

- Rust workspace;
- crate `jpegli-sys` для FFI и сборки upstream JPEGli;
- crate `jpegli` для safe Rust API;
- vendored/pinned исходники upstream JPEGli;
- минимальный рабочий пример использования;
- unit tests и smoke/golden tests;
- CI-конфигурацию;
- README с описанием сборки, лицензирования и примерами;
- зафиксированный upstream commit, явно указанный в документации;
- все необходимые license/notice файлы для vendored upstream.

3. Архитектура

Рекомендуемая структура workspace:

workspace/
  Cargo.toml
  crates/
    jpegli-sys/
    jpegli/
  vendor/
    jpegli/

3.1. Crate `jpegli-sys`

`jpegli-sys` отвечает за:

- сборку upstream JPEGli;
- FFI-слой (`extern "C"`);
- raw types / constants / bindings;
- линковку;
- C shim для безопасного моста ошибок;
- экспорт низкоуровневого API.

3.2. Crate `jpegli`

`jpegli` отвечает за:

- safe API;
- владение памятью и безопасную работу с буферами;
- проверку конфигурации;
- преобразование ошибок в Rust `Result`;
- высокоуровневые Rust-типы конфигурации;
- удобный encode API.

4. Требования к upstream и лицензированию

Обязательно:

- использовать official Google JPEGli;
- upstream должен быть pinned на конкретный commit;
- автоматическое скачивание upstream из сети во время `cargo build` запрещено;
- исходники upstream должны поставляться локально в составе проекта;
- license/notice upstream должны быть включены в репозиторий и пакет;
- AGPL-зависимости запрещены.

5. Политика сборки

5.1. Обязательные режимы сборки

`jpegli-sys` должен поддерживать два режима:

А. `vendored` — режим по умолчанию

- сборка JPEGli из `vendor/jpegli`;
- без сетевых обращений во время сборки;
- воспроизводимая сборка.

Б. `system` — опциональный режим

- линковка к уже установленной системной библиотеке;
- поиск через `pkg-config` и/или env-переменные.

5.2. Требования к build.rs

`build.rs` должен:

- находить `vendor/jpegli`;
- проверять наличие необходимых исходников upstream;
- вызывать CMake;
- собирать библиотечный target, а не весь проект целиком;
- по возможности собирать именно `jpegli-static`;
- не собирать CLI tools (`cjpegli`, `djpegli`), benchmarks, docs, tests;
- линковать статически по умолчанию;
- поддерживать env overrides.

6. Env overrides

Должны поддерживаться следующие переменные окружения:

- `JPEGLI_SYS_USE_SYSTEM=1`
- `JPEGLI_SYS_ROOT=...`
- `JPEGLI_SYS_STATIC=1`
- `JPEGLI_SYS_CMAKE_TOOLCHAIN_FILE=...`

При необходимости допускается добавить дополнительные переменные, если они документированы.

7. Стратегия bindings

7.1. Требования

По умолчанию не запускать `bindgen` у конечного пользователя.

7.2. Реализация

Должно быть реализовано следующее:

- bindings генерируются заранее;
- generated bindings коммитятся в репозиторий;
- для мейнтейнера допускается отдельный feature/режим `generate-bindings`;
- обычная сборка пользователя не должна требовать установленного `libclang`.

8. Error model и C shim

8.1. Требование

Обязателен отдельный C/C++ shim для локализации libjpeg-style error handling.

8.2. Причина

Safe Rust API не должен зависеть от прямого прохождения аварийного control flow через C-границу.

8.3. Реализация

C shim должен:

- создавать и инициализировать error manager;
- хранить буфер сообщения об ошибке;
- локализовать `setjmp/longjmp`-based error path на C-стороне;
- оборачивать жизненный цикл encoder state;
- возвращать наружу status code + error message;
- предоставлять простой и узкий интерфейс для Rust-слоя.

8.4. Требование к Rust API

На стороне `jpegli` наружу должен возвращаться нормальный `Result<T, Error>`.

9. Scope v0.1

В рамках v0.1 обязательно реализовать только encode path.

9.1. Что входит в v0.1

- encode RGB8 из памяти в память;
- encode RGBA8 из памяти в память;
- encode Gray8 из памяти в память;
- mem destination (`Vec<u8>`);
- выбор качества через:
  - `Quality(u8)`
  - `Distance(f32)`
- progressive on/off;
- subsampling:
  - `Auto`
  - `444`
  - `422`
  - `420`
- optional ICC profile pass-through;
- optional optimize coding flag;
- optional baseline compatibility flag;
- корректная обработка stride;
- безопасная валидация входных параметров;
- thread-safe публичный API на уровне Rust-объектов при условии отсутствия совместного использования одного encoder instance между потоками.

9.2. Что не входит в v0.1

Следующие возможности исключены из scope v0.1:

- decode API;
- coefficient-level API;
- raw MCU/raw DCT API;
- marker processor callbacks;
- streaming writer;
- custom memory manager;
- safe API для `jpegli_set_psnr`;
- XYB и advanced transfer functions в high-level safe API;
- wasm/mobile support;
- сложная многопоточная внутренняя архитектура.

10. Публичный safe API

Требуемая форма high-level API:

pub struct Encoder {
    cfg: EncoderConfig,
}

pub struct EncoderConfig {
    pub quality: Option`<u8>`,
    pub distance: Option`<f32>`,
    pub progressive: bool,
    pub subsampling: ChromaSubsampling,
    pub optimize_coding: bool,
    pub icc_profile: Option<Vec`<u8>`>,
}

pub enum ChromaSubsampling {
    Auto,
    Cs444,
    Cs422,
    Cs420,
}

pub struct ImageView<'a> {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub stride: usize,
    pub data: &'a [u8],
}

pub enum PixelFormat {
    Rgb8,
    Rgba8,
    Gray8,
}

impl Encoder {
    pub fn new(cfg: EncoderConfig) -> Result<Self, Error>;
    pub fn encode(&self, image: &ImageView<'_>) -> Result<Vec`<u8>`, Error>;
}

10.1. Правила API

Обязательно:

- нельзя одновременно задавать `quality` и `distance`;
- если заданы и `quality`, и `distance`, должна возвращаться ошибка конфигурации;
- конфигурация должна валидироваться до начала encode;
- `encode()` не должен приводить к UB при некорректных входных параметрах;
- для JPEG alpha по умолчанию отбрасывается;
- поведение RGBA должно быть явно задокументировано.

Допускается в будущем расширение API, но без ломающего redesign базового интерфейса.

11. Модель ошибок Rust

Ожидаемая форма high-level ошибок:

pub enum Error {
    InvalidConfig(&'static str),
    InvalidImage(&'static str),
    EncodeFailed(String),
    NullPointer,
    Internal(&'static str),
}

Допускаются изменения деталей `Error`, если сохраняется следующая семантика:

- ошибки конфигурации отделены от ошибок encode;
- ошибки входного буфера отделены от внутренних ошибок;
- внутренние ошибки JPEGli не теряются и могут быть диагностированы.

12. Политика по advanced API

12.1. В `jpegli-sys`

Допускается экспорт raw bindings шире, чем используется в `jpegli`.

12.2. В `jpegli`

Не требуется поднимать в safe high-level API в v0.1:

- `jpegli_set_psnr`
- coefficient read/write
- raw data paths
- custom marker processors
- experimental input/output format knobs, не нужные для базового encode-first сценария

Цель — сохранить возможность расширения, не усложняя v0.1.

13. Cargo features

13.1. Для `jpegli-sys`

Должны быть предусмотрены:

- `vendored` — default
- `system`
- `static` — default
- `shared`
- `generate-bindings` — off by default

13.2. Для `jpegli`

Допускается/рекомендуется предусмотреть:

- `icc`
- `rgb`
- `image`
- `rayon` — только если действительно используется и документирован

14. Поддерживаемые платформы

14.1. Обязательная поддержка v0.1

- Linux x86_64
- Linux aarch64
- macOS arm64
- macOS x86_64
- Windows x86_64-msvc

14.2. Не требуется в v0.1

- musl
- Android
- iOS
- wasm

15. Тесты

15.1. Unit tests

Обязательно покрыть:

- invalid config;
- invalid dimensions;
- empty buffer;
- wrong stride;
- rgb encode success;
- rgba encode with alpha drop;
- gray encode success;
- mutual exclusion of `quality` and `distance`.

15.2. Golden / smoke tests

Обязательно:

- кодирование известного fixture;
- проверка, что результат открывается стандартным JPEG-декодером;
- проверка размеров и базовых свойств выходного изображения;
- smoke-check, что выходной JPEG не повреждён.

15.3. Differential / stability tests

Желательно:

- сравнение encode при `quality`;
- сравнение encode при `distance`;
- проверка стабильности поведения в рамках pinned upstream commit.

16. CI

Должен быть настроен CI минимум для:

- Linux
- macOS
- Windows

Минимальный набор job:

- `cargo build`
- `cargo test`
- `cargo clippy`
- `cargo doc --no-deps`

Желательно:

- отдельная матрица для `vendored`;
- отдельная проверка `system` хотя бы на Linux.

17. Документация

README должен содержать:

- описание назначения проекта;
- указание, что это wrapper над official Google JPEGli;
- указание pinned upstream commit;
- описание режимов `vendored` и `system`;
- описание того, что default mode собирает vendored source;
- описание safe API;
- описание ограничений v0.1;
- объяснение, что `distance` — preferred modern quality knob;
- пояснение, что `psnr` в high-level API намеренно не expose’ится в v0.1;
- минимальный пример использования;
- описание лицензий.

18. Non-goals

Вне scope текущей работы:

- реализация decode API;
- полноценный universal wrapper над всем API JPEGli;
- поддержка всех advanced/external JPEGli knobs в high-level Rust API;
- сетевые загрузки зависимостей в `build.rs`;
- интеграция с AGPL-проектами;
- оптимизация под все платформы и toolchains вне списка supported platforms.

19. Критерии приёмки

Работа считается принятой, если одновременно выполнены все условия:

- проект собирается в default-конфигурации без сетевых обращений;
- `vendored` режим работает;
- safe encode API реализован и документирован;
- `Encoder::encode()` корректно возвращает `Vec<u8>` для валидного RGB/RGBA/Gray ввода;
- ошибки конфигурации и ошибки encode возвращаются как `Result::Err`, без UB;
- C shim реализован и используется;
- имеются unit tests и smoke/golden tests;
- CI настроен и проходит на поддерживаемых платформах;
- README и лицензии оформлены;
- upstream JPEGli pinned на конкретный commit и это явно задокументировано.

20. Дополнительные требования к качеству реализации

- избегать лишнего unsafe в high-level crate;
- unsafe должен быть локализован преимущественно в `jpegli-sys` и shim;
- код должен быть структурирован так, чтобы decode API можно было добавить позже без полного redesign;
- публичный API должен быть небольшим, ясным и стабильным;
- поведение при некорректных входных данных должно быть предсказуемым и документированным.
