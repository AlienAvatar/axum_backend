use reqwest;
use scraper::{Html, Selector};
use crate::article::schema::CreateArticleSchema;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse},
    Json,
};
use std::{collections::HashMap, sync::Arc};
use crate::db::DB;
use crate::error::MyError;

pub async fn public_handler(document:Html, app_state: Arc<AppState>){
     // 使用 CSS 选择器定位目标元素
     let link_selector = Selector::parse("li.list-cat-title > a").unwrap();
     let link_elements = document.select(&link_selector);

      // 遍历找到的元素,提取 href 属性值并访问
     for link in link_elements {
        let title = link.text().collect::<Vec<_>>().join(" ");
        
        // 获取 href 属性值
        if let Some(href) = link.value().attr("href") {
            println!("Visiting: {}", href);
            visit_link(href, title, app_state.clone(),"公告").await;
        }
    }
}

pub async fn buddha_handler(document:Html, app_state: Arc<AppState>){
    // 使用 CSS 选择器定位目标元素
    let link_selector = Selector::parse(".entry-title > a").unwrap();
    let link_elements = document.select(&link_selector);

     // 遍历找到的元素,提取 href 属性值并访问
    for link in link_elements {
       let title = link.text().collect::<Vec<_>>().join(" ");
       
       // 获取 href 属性值
       if let Some(href) = link.value().attr("href") {
           println!("Visiting: {}", href);
           visit_link(href, title, app_state.clone(), "古佛降世").await;
       }
   }
}

pub async fn recognition_handler(document:Html, app_state: Arc<AppState>){
    // 使用 CSS 选择器定位目标元素
    let link_selector = Selector::parse(".entry-title > a").unwrap();
    let link_elements = document.select(&link_selector);

    // 遍历找到的元素,提取 href 属性值并访问
    for link in link_elements {
       let title = link.text().collect::<Vec<_>>().join(" ");
       
       // 获取 href 属性值
       if let Some(href) = link.value().attr("href") {
           println!("Visiting: {}", href);
           visit_link(href, title, app_state.clone(), "认证恭贺").await;
       }
    }

    //当前的页数靠手动控制
    let page_selector = Selector::parse(".page-numbers").unwrap();
    let page_numbers: Vec<String> = document
        .select(&page_selector)
        .map(|element: scraper::ElementRef| element.text().collect())
        .collect();
    let page_total = page_numbers.len() - 1;
    let page_elements = document.select(&page_selector).take(page_total);

    for page in page_elements {
        // 获取 page href 属性值
        if let Some(page_href) = page.value().attr("href") {
            println!("Visiting: {}", page_href);
            // 访问链接并获取响应
            let page_response = reqwest::get(page_href).await.unwrap();
            let page_html = page_response.text().await.unwrap();

            let page_document = Html::parse_document(&page_html);
            let page_link_selector = Selector::parse(".entry-title > a").unwrap();
            let page_link_elements = page_document.select(&page_link_selector);

            for link in page_link_elements {
                let title = link.text().collect::<Vec<_>>().join(" ");
                
                // 获取 href 属性值
                if let Some(href) = link.value().attr("href") {
                    println!("Visiting: {}", href);
                    visit_link(href, title, app_state.clone(), "认证恭贺").await;
                }
            }
        }
    }

    // 当前的页数靠手动控制
    // let mut page_numbers: Vec<String> = document
    //     .select(&page_selector)
    //     .take(3)
    //     .map(|element| element.text().collect())
    //     .collect();

    // println!("{:?}", page_numbers);
}

//访问公告
async fn visit_public_link(url: &str, title: String, app_state: Arc<AppState>) 
{
    // 访问链接并获取响应
    let response = reqwest::get(url).await.unwrap();
    let html = response.text().await.unwrap();

    let document = Html::parse_document(&html);
    let single_content_selector = Selector::parse(".single-content >p").unwrap();
    let single_content_elements = document.select(&single_content_selector);
    let mut count = 0;

    let mut content: String = "".to_string();
    for single_content_element in single_content_elements {
        single_content_element.value();
        let str = single_content_element.inner_html();
        let format_p = format!("<p>{}</p>", str);
        content += &format_p;
        
    }
    println!("{}", content);

    let crawler_body = CreateArticleSchema{
        title: title.clone(),
        content: content,
        author: "管理员".to_string(),
        category: "公告".to_string(),
    };
    
    match app_state
        .db
        .create_article(&crawler_body)
        .await.map_err(MyError::from) 
    {
        Ok(res) => {
            count += 1;
            println!("count {}", count);
            println!("{}", res.data.article.id.to_string());
        },
        Err(e) => {
            println!("{}", e.to_string());
        },
    }
}

async fn visit_link(url: &str, title: String, app_state: Arc<AppState>, category_str: &str){
    // 访问链接并获取响应
    let response = reqwest::get(url).await.unwrap();
    let html = response.text().await.unwrap();

    let document = Html::parse_document(&html);
    let single_content_selector = Selector::parse(".single-content >p").unwrap();
    let single_content_elements = document.select(&single_content_selector);
    let mut count = 0;

    let mut content: String = "".to_string();
    for single_content_element in single_content_elements {
        single_content_element.value();
        let str = single_content_element.inner_html();
        let format_p = format!("<p>{}</p>", str);
        content += &format_p;
        
    }
    println!("{}", content);

    let crawler_body = CreateArticleSchema{
        title: title.clone(),
        content: content,
        author: "管理员".to_string(),
        category: category_str.to_string(),
    };
    
    match app_state
        .db
        .create_article(&crawler_body)
        .await.map_err(MyError::from) 
    {
        Ok(res) => {
            count += 1;
            println!("count {}", count);
            println!("{}", res.data.article.id.to_string());
        },
        Err(e) => {
            println!("{}", e.to_string());
        },
    }
}

pub async fn buddha_dharma_handler(document:Html, app_state: Arc<AppState>){
    // 使用 CSS 选择器定位目标元素
    let link_selector = Selector::parse(".entry-title > a").unwrap();
    let link_elements = document.select(&link_selector);

     // 遍历找到的元素,提取 href 属性值并访问
    for link in link_elements {
       let title = link.text().collect::<Vec<_>>().join(" ");
       
       // 获取 href 属性值
       if let Some(href) = link.value().attr("href") {
           println!("Visiting: {}", href);
           visit_link(href, title, app_state.clone(), "羌佛说法").await;
       }
   }
}

pub async fn holy_realization_handler(document:Html, app_state: Arc<AppState>){
    // 使用 CSS 选择器定位目标元素
    let link_selector = Selector::parse(".entry-title > a").unwrap();
    let link_elements = document.select(&link_selector);

    // 遍历找到的元素,提取 href 属性值并访问
    for link in link_elements {
       let title = link.text().collect::<Vec<_>>().join(" ");
       
       // 获取 href 属性值
       if let Some(href) = link.value().attr("href") {
           println!("Visiting: {}", href);
           visit_link(href, title, app_state.clone(), "羌佛圣量").await;
       }
    }

    //当前的页数靠手动控制
    let page_selector = Selector::parse(".page-numbers").unwrap();
    let page_numbers: Vec<String> = document
        .select(&page_selector)
        .map(|element: scraper::ElementRef| element.text().collect())
        .collect();
    let page_total = page_numbers.len() - 1;
    let page_elements = document.select(&page_selector).take(page_total);

    for page in page_elements {
        // 获取 page href 属性值
        if let Some(page_href) = page.value().attr("href") {
            println!("Visiting: {}", page_href);
            // 访问链接并获取响应
            let page_response = reqwest::get(page_href).await.unwrap();
            let page_html = page_response.text().await.unwrap();

            let page_document = Html::parse_document(&page_html);
            let page_link_selector = Selector::parse(".entry-title > a").unwrap();
            let page_link_elements = page_document.select(&page_link_selector);

            for link in page_link_elements {
                let title = link.text().collect::<Vec<_>>().join(" ");
                
                // 获取 href 属性值
                if let Some(href) = link.value().attr("href") {
                    println!("Visiting: {}", href);
                    visit_link(href, title, app_state.clone(), "羌佛圣量").await;
                }
            }
        }
    }
}

pub async fn holy_occurrences_handler(document:Html, app_state: Arc<AppState>){
    // 使用 CSS 选择器定位目标元素
    let link_selector = Selector::parse(".entry-title > a").unwrap();
    let link_elements = document.select(&link_selector);

    // 遍历找到的元素,提取 href 属性值并访问
    for link in link_elements {
       let title = link.text().collect::<Vec<_>>().join(" ");
       
       // 获取 href 属性值
       if let Some(href) = link.value().attr("href") {
           println!("Visiting: {}", href);
           visit_link(href, title, app_state.clone(), "羌佛圣迹").await;
       }
    }

    let page_selector = Selector::parse(".page-numbers").unwrap();
    
    let page_numbers: Vec<String> = document
    .select(&page_selector)
    .map(|element: scraper::ElementRef| element.text().collect())
    .collect();
    let page_total = page_numbers.len() - 1;
    println!("page_numbers {:?}", page_numbers.len()-1);
    let page_elements = document.select(&page_selector).take(page_total);

    for page in page_elements {
        // 获取 page href 属性值
        if let Some(page_href) = page.value().attr("href") {
            println!("Visiting: {}", page_href);
            // 访问链接并获取响应
            let page_response = reqwest::get(page_href).await.unwrap();
            let page_html = page_response.text().await.unwrap();

            let page_document = Html::parse_document(&page_html);
            let page_link_selector = Selector::parse(".entry-title > a").unwrap();
            let page_link_elements = page_document.select(&page_link_selector);

            for link in page_link_elements {
                let title = link.text().collect::<Vec<_>>().join(" ");
                
                // 获取 href 属性值
                if let Some(href) = link.value().attr("href") {
                    println!("Visiting: {}", href);
                    visit_link(href, title, app_state.clone(), "羌佛圣迹").await;
                }
            }
        }
    }
}

pub async fn public_crawl_handler(document:Html, app_state: Arc<AppState>, category: &str){
    // 使用 CSS 选择器定位目标元素
    let link_selector = Selector::parse(".entry-title > a").unwrap();
    let link_elements = document.select(&link_selector);

    // 遍历找到的元素,提取 href 属性值并访问
    for link in link_elements {
       let title = link.text().collect::<Vec<_>>().join(" ");
       
       // 获取 href 属性值
       if let Some(href) = link.value().attr("href") {
           println!("Visiting: {}", href);
           visit_link(href, title, app_state.clone(), category).await;
       }
    }

    let page_selector = Selector::parse(".page-numbers").unwrap();
    let page_numbers: Vec<String> = document
        .select(&page_selector)
        .map(|element: scraper::ElementRef| element.text().collect())
        .collect();
    if(page_numbers.len() == 0){
        return;
    }
    let page_total = page_numbers.len() - 1;
    println!("page_numbers {:?}", page_numbers.len()-1);
    let page_elements = document.select(&page_selector).take(page_total);

    for page in page_elements {
        // 获取 page href 属性值
        if let Some(page_href) = page.value().attr("href") {
            println!("Visiting: {}", page_href);
            // 访问链接并获取响应
            let page_response = reqwest::get(page_href).await.unwrap();
            let page_html = page_response.text().await.unwrap();

            let page_document = Html::parse_document(&page_html);
            let page_link_selector = Selector::parse(".entry-title > a").unwrap();
            let page_link_elements = page_document.select(&page_link_selector);

            for link in page_link_elements {
                let title = link.text().collect::<Vec<_>>().join(" ");
                
                // 获取 href 属性值
                if let Some(href) = link.value().attr("href") {
                    println!("Visiting: {}", href);
                    visit_link(href, title, app_state.clone(), category).await;
                }
            }
        }
    }
}