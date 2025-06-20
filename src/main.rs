pub mod domain;
pub mod service;

use domain::DocumentPreparer;

fn main() {
    let text = "
        Мороз и солнце; день чудесный!
        Еще ты дремлешь, друг прелестный —
        Пора, красавица, проснись:
        Открой сомкнуты негой взоры
        Навстречу северной Авроры,
        Звездою севера явись!

        Вечор, ты помнишь, вьюга злилась,
        На мутном небе мгла носилась;
        Луна, как бледное пятно,
        Сквозь тучи мрачные желтела,
        И ты печальная сидела —
        А нынче… погляди в окно:";

//     let document = domain::Document::new(text.to_string());
//     let service = service::DocumentService::new(256);
//     let chunks = service.prepare(&document);

//     for chunk in &chunks {
//         println!("{:?}", chunk);
//     }
}
