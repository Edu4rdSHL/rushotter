use {
    constants::{CHROMEDRIVER_URL, GECKODRIVER_URL},
    futures::StreamExt,
    std::{
        collections::HashMap,
        path::{Path, PathBuf},
    },
    thirtyfour::prelude::*,
};

mod constants;

pub async fn take_chrome_screenshots(
    driver: Option<WebDriver>,
    chrome_args: Option<&[&str]>,
    // The first argument is the URL to take a screenshot of and the second argument is the path to save the screenshot to.
    // If the path is not provided, the screenshot will be saved to the current directory in the format of "url.png".
    websites: HashMap<String, Option<String>>,
    threads: Option<usize>,
) -> WebDriverResult<()> {
    // Set the number of threads to use. Default to 5.
    let threads = if let Some(threads) = threads {
        threads
    } else {
        5
    };

    // Create a new ChromeCapabilities instance.
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;

    // Add any additional Chrome command line arguments.
    if let Some(chrome_args) = chrome_args {
        for arg in chrome_args {
            caps.add_chrome_arg(arg)?;
        }
    }

    // Create a new WebDriver instance.
    let driver = if let Some(driver) = driver {
        driver
    } else {
        WebDriver::new(CHROMEDRIVER_URL, caps).await?
    };

    println!("Taking screenshots...");

    take_screenshots(websites, &driver, threads).await;

    Ok(())
}

pub async fn take_firefox_screenshots(
    driver: Option<WebDriver>,
    firefox_args: Option<Vec<&str>>,
    // The first argument is the URL to take a screenshot of and the second argument is the path to save the screenshot to.
    // If the path is not provided, the screenshot will be saved to the current directory in the format of "url.png".
    websites: HashMap<String, Option<String>>,
    threads: usize,
) -> WebDriverResult<()> {
    let mut caps = DesiredCapabilities::firefox();
    caps.set_headless()?;

    if let Some(firefox_args) = firefox_args {
        for arg in firefox_args {
            caps.add_firefox_arg(arg)?;
        }
    }

    let driver = if let Some(driver) = driver {
        driver
    } else {
        WebDriver::new(GECKODRIVER_URL, caps).await?
    };

    take_screenshots(websites, &driver, threads).await;

    if driver.quit().await.is_err() {
        println!("Failed to quit driver");
    }

    Ok(())
}

pub async fn take_screenshots(
    websites: HashMap<String, Option<String>>,
    driver: &WebDriver,
    threads: usize,
) {
    futures::stream::iter(websites.into_iter().map(|(website, screenshot_path)| {
        let path = if let Some(screenshot_path) = screenshot_path {
            Path::new(&screenshot_path).with_extension("png")
        } else {
            Path::new(&website).with_extension("png")
        };

        let fut = screenshot(driver.clone(), website, path);

        tokio::spawn(fut)
    }))
    .buffer_unordered(threads)
    .collect::<Vec<_>>()
    .await;
}

pub async fn screenshot(driver: WebDriver, website: String, path: PathBuf) {
    if driver.goto(&website).await.is_err() || driver.screenshot(&path).await.is_err() {
        println!("Failed to take screenshot of {}", website);
    }
}
