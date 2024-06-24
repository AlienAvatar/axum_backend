use reqwest;
use scraper::{Html, Selector};

pub async fn public_handler(document:Html){
     // 使用 CSS 选择器定位目标元素
     let link_selector = Selector::parse("li.list-cat-title > a").unwrap();
     let link_elements = document.select(&link_selector);
  
      // 遍历找到的元素,提取 href 属性值并访问
     for link in link_elements {
          if let Some(href) = link.value().attr("href") {
              println!("Visiting: {}", href);
              visit_public_link(href).await;
          }
     }
}

//访问公告
async fn visit_public_link(url: &str) {
    // 访问链接并获取响应
    let response = reqwest::get(url).await.unwrap();
    let html = response.text().await.unwrap();

    let document = Html::parse_document(&html);
    let single_content_selector = Selector::parse(".single-content >p").unwrap();
    let single_content_elements = document.select(&single_content_selector);

    for single_content_element in single_content_elements {
        single_content_element.value();
        let str = single_content_element.inner_html();
        let format_p = format!("<p>{}</p>", str);
        println!("{}", format_p);

        let crawler_body = CreateArticleSchema{
            ..body
        };

        match app_state
            .db
            .create_article(&crawler_body)
            .await.map_err(MyError::from) 
        {
            Ok(res) => Ok((StatusCode::CREATED, Json(res))),
            Err(e) => Err(e.into()),
        }

    }
}