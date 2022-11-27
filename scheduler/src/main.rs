use async_std::task;
use job_scheduler::Job;
use job_scheduler::JobScheduler;
use models::Category;
use models::ToMoneyReplenishment;
use models::ToSchedule;
use notion::document::Document;
use notion::post_new_notion_document;
use notion::NotionApi;
use notion::DATABASE_ID;

fn main() {
    dotenv::dotenv().ok();

    pretty_env_logger::init();

    let mut sched = JobScheduler::new();

    let categories = [
        Category::FoodAndDrinks,
        Category::Rent,
        Category::UkrReponsibilities,
        Category::OperationalSpends,
    ];
    let mut jobs = Vec::new();
    categories.into_iter().for_each(|category| {
        let schedules = category.replenish_money_schedules();
        schedules.into_iter().for_each(|schedule| {
            let job = Job::new(schedule.parse().unwrap(), move || {
                log::info!("Executing the job for {}", category.to_string());
                let replenishment_document = Document::new(DATABASE_ID.to_owned())
                    .convert_to_replenishment(&category, category.to_replenishment_amount());
                task::block_on(post_new_notion_document(
                    &NotionApi::from_env(),
                    replenishment_document,
                ));
            });
            jobs.push(job);
        });
    });

    for job in jobs.into_iter() {
        sched.add(job);
    }

    loop {
        sched.tick();

        std::thread::sleep(sched.time_till_next_job());
    }
}
