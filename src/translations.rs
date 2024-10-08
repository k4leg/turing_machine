// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt;

use fluent::{FluentBundle, FluentResource};
use sys_locale::get_locale;
use unic_langid::LanguageIdentifier;

#[derive(Clone)]
pub enum AppLanguage {
    English,
    Russian,
}

impl AppLanguage {
    const ENG: &str = "en-US";
    const RUS: &str = "ru-RU";

    const FTL_EN: &str = r#"
zoom = Zoom
alphabet-primary = Primary Alphabet
alphabet-secondary = Secondary Alphabet
input = Input
command-add = Add command
command-remove = Remove command
tape-add = Add tape
tape-remove = Remove tape
stop = Stop
start = Start
protocol-save = Save protocol
ok-file-saved = The file was saved successfully
plotting-stop = Stop plotting
plotting-start = Start plotting
label-presets = presets
line-complexity = Complexity
axis-length-of-number = Length of Word
axis-max-steps = Max Steps
err-no-protocol = No protocol
err-no-path-given = No path given
err-failed-to-create-open = Failed to create/open:
err-failed-to-write = Failed to write to
col-state = State
col-cell = Cell
col-dir = Dir
col-protocol = Protocol
btn-change-language = Change Language
label-number-sign = #
"#;

    const FTL_RU: &str = r#"
zoom = Масштаб
alphabet-primary = Основной алфавит
alphabet-secondary = Дополнительный алфавит
input = Ввод
command-add = Добавить команду
command-remove = Удалить команду
tape-add = Добавить ленту
tape-remove = Удалить ленту
stop = Стоп
start = Старт
protocol-save = Сохранить протокол
ok-file-saved = Файл был сохранён успешно
plotting-stop = Остановить построение графика
plotting-start = Начать построение графика
label-presets = пресеты
line-complexity = Сложность
axis-length-of-number = Длина слова
axis-max-steps = Максимальное количество шагов
err-no-protocol = Нет протокола
err-no-path-given = Путь не задан
err-failed-to-create-open = Не удалость создать/открыть:
err-failed-to-write = Не удалось записать в
col-state = Сост.
col-cell = Ячейка
col-dir = Направ.
col-protocol = Протокол
btn-change-language = Сменить язык
label-number-sign = №
"#;

    pub fn build_or_default(s: &str) -> Self {
        if s == Self::RUS {
            Self::Russian
        } else {
            Self::English
        }
    }

    pub fn get_ftl(&self) -> &'static str {
        match self {
            AppLanguage::English => Self::FTL_EN,
            AppLanguage::Russian => Self::FTL_RU,
        }
    }

    pub fn get_lang_id(&self) -> LanguageIdentifier {
        self.to_string().parse().unwrap()
    }

    pub fn get_res(&self) -> FluentResource {
        FluentResource::try_new(self.get_ftl().into()).unwrap()
    }

    pub fn get_bundle(&self) -> FluentBundle<FluentResource> {
        let mut bundle = FluentBundle::new(vec![self.get_lang_id()]);
        bundle.add_resource(self.get_res()).unwrap();
        bundle
    }

    pub fn next(&self) -> Self {
        match self {
            Self::English => Self::Russian,
            Self::Russian => Self::English,
        }
    }
}

impl Default for AppLanguage {
    fn default() -> Self {
        let lang = get_locale().unwrap_or_else(|| Self::ENG.to_string());
        Self::build_or_default(&lang)
    }
}

impl fmt::Display for AppLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppLanguage::English => Self::ENG.fmt(f),
            AppLanguage::Russian => Self::RUS.fmt(f),
        }
    }
}
