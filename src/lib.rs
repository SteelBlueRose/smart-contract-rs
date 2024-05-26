use schemars::JsonSchema;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId};
use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AccountIdWrapper(AccountId);

impl JsonSchema for AccountIdWrapper {
    fn schema_name() -> String {
        "AccountIdWrapper".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <String>::json_schema(gen)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct AccountRewardPoints {
    owner: AccountIdWrapper,
    points: i64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct Task {
    id: u64,
    title: String,
    description: String,
    priority: u8,
    deadline: Option<u64>,
    estimated_time: Option<f64>,
    reward_points: i64,
    completed: bool,
    preferred_start_time: Option<f64>,
    preferred_end_time: Option<f64>,
    owner: AccountIdWrapper,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct Reward {
    id: u64,
    title: String,
    description: String,
    cost: i64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct WorkingHours {
    start_time: f64,
    end_time: f64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct WeeklyWorkingHours {
    monday: WorkingHours,
    tuesday: WorkingHours,
    wednesday: WorkingHours,
    thursday: WorkingHours,
    friday: WorkingHours,
    saturday: WorkingHours,
    sunday: WorkingHours,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct TimeSlot {
    start_time: f64,
    end_time: f64,
    task_id: Option<u64>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub struct TimeSlots {
    monday: Vec<TimeSlot>,
    tuesday: Vec<TimeSlot>,
    wednesday: Vec<TimeSlot>,
    thursday: Vec<TimeSlot>,
    friday: Vec<TimeSlot>,
    saturday: Vec<TimeSlot>,
    sunday: Vec<TimeSlot>,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct TodoListV1 {
    tasks: HashMap<AccountId, Vec<Task>>,
    rewards: HashMap<AccountId, Vec<Reward>>,
    account_reward_points: HashMap<AccountId, i64>,
    completed_tasks_per_day: HashMap<AccountId, HashMap<String, u32>>,
    working_hours: HashMap<AccountId, WeeklyWorkingHours>,
    time_slots: HashMap<AccountId, TimeSlots>,
}

#[near_bindgen]
impl TodoListV1 {
    #[init]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task(&mut self, title: String, description: String, priority: u8, 
                    deadline: Option<u64>, estimated_time: Option<f64>, reward_points: i64,
                    preferred_start_time: Option<f64>, preferred_end_time: Option<f64>) {
        let account_id = env::signer_account_id();
        

        let task = Task {
            id: self.tasks.get(&account_id).map_or(1, |tasks| tasks.len() as u64 + 1),
            title,
            description,
            priority,
            deadline,
            estimated_time,
            reward_points,
            completed: false,
            preferred_start_time,
            preferred_end_time,
            owner: AccountIdWrapper(account_id.clone()),
        };
        self.tasks.entry(account_id).or_insert_with(Vec::new).push(task);
    }

    pub fn remove_task(&mut self, id: u64) {
        let account_id = env::signer_account_id();
        

        if let Some(tasks) = self.tasks.get_mut(&account_id) {
            tasks.retain(|task| task.id != id);
        }
    }

    pub fn get_tasks(&self, account_id: AccountId) -> Vec<Task> {
        
        self.tasks.get(&account_id).cloned().unwrap_or_default()
    }

    pub fn update_task(&mut self, id: u64, title: String, description: String, priority: u8, 
                       deadline: Option<u64>, estimated_time: Option<f64>, reward_points: i64,
                       preferred_start_time: Option<f64>, preferred_end_time: Option<f64>) {
        let account_id = env::signer_account_id();
        

        if let Some(tasks) = self.tasks.get_mut(&account_id) {
            if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
                task.title = title;
                task.description = description;
                task.priority = priority;
                task.deadline = deadline;
                task.estimated_time = estimated_time;
                task.reward_points = reward_points;
                task.preferred_start_time = preferred_start_time;
                task.preferred_end_time = preferred_end_time;
            }
        }
    }

    pub fn mark_complete(&mut self, id: u64) {
        let account_id = env::signer_account_id();
        

        if let Some(tasks) = self.tasks.get_mut(&account_id) {
            if let Some(task) = tasks.iter_mut().find(|task| task.id == id) {
                task.completed = true;
                let reward_points = task.reward_points;
                let account_points = self.account_reward_points.entry(account_id.clone()).or_insert(0);
                *account_points = (*account_points + reward_points).max(0);
                self.update_completed_tasks_per_day(&account_id);
            }
        }
    }

    pub fn add_reward(&mut self, title: String, description: String, cost: i64) {
        let account_id = env::signer_account_id();

        let reward = Reward {
            id: self.rewards.get(&account_id).map_or(1, |rewards| rewards.len() as u64 + 1),
            title,
            description,
            cost,
        };
        self.rewards.entry(account_id).or_insert_with(Vec::new).push(reward);
    }

    pub fn get_rewards(&self, account_id: AccountId) -> Vec<Reward> {
        
        self.rewards.get(&account_id).cloned().unwrap_or_default()
    }

    pub fn remove_reward(&mut self, id: u64) {
        let account_id = env::signer_account_id();
        

        if let Some(rewards) = self.rewards.get_mut(&account_id) {
            rewards.retain(|reward| reward.id != id);
        }
    }

    pub fn redeem_reward(&mut self, id: u64) -> bool {
        let account_id = env::signer_account_id();
        

        if let Some(rewards) = self.rewards.get(&account_id) {
            if let Some(reward) = rewards.iter().find(|reward| reward.id == id) {
                let account_points = self.account_reward_points.entry(account_id.clone()).or_insert(0);
                if *account_points >= reward.cost {
                    *account_points -= reward.cost;
                    return true;
                }
            }
        }
        false
    }

    pub fn get_account_reward_points(&self, account_id: AccountId) -> i64 {
        
        *self.account_reward_points.get(&account_id).unwrap_or(&0)
    }

    fn update_completed_tasks_per_day(&mut self, account_id: &AccountId) {

        let today = env::block_timestamp() / 86400000000000;
        let date_string = format!("{}", today);

        let user_tasks = self.completed_tasks_per_day.entry(account_id.clone()).or_insert_with(HashMap::new);
        let count = user_tasks.entry(date_string).or_insert(0);
        *count += 1;
    }

    pub fn get_completed_tasks_per_day(&self, account_id: AccountId) -> HashMap<String, u32> {
        
        self.completed_tasks_per_day.get(&account_id).cloned().unwrap_or_default()
    }

    pub fn get_working_hours(&self, account_id: AccountId) -> WeeklyWorkingHours {
        
        self.working_hours.get(&account_id).cloned().unwrap_or_else(|| WeeklyWorkingHours {
            monday: WorkingHours { start_time: 9.0, end_time: 16.0 },
            tuesday: WorkingHours { start_time: 9.0, end_time: 16.0 },
            wednesday: WorkingHours { start_time: 9.0, end_time: 16.0 },
            thursday: WorkingHours { start_time: 9.0, end_time: 16.0 },
            friday: WorkingHours { start_time: 9.0, end_time: 16.0 },
            saturday: WorkingHours { start_time: 9.0, end_time: 16.0 },
            sunday: WorkingHours { start_time: 9.0, end_time: 16.0 },
        })
    }

    pub fn update_working_hours(&mut self, working_hours: WeeklyWorkingHours) {
        let account_id = env::signer_account_id();
        
        self.working_hours.insert(account_id, working_hours);
    }

    pub fn get_time_slots(&self, account_id: AccountId) -> Option<TimeSlots> {
        
        self.time_slots.get(&account_id).cloned()
    }

    pub fn update_time_slots(&mut self, time_slots: TimeSlots) {
        let account_id = env::signer_account_id();
        
        self.time_slots.insert(account_id, time_slots);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_add_task() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();
        
        contract.add_task(
            "Test Task".to_string(),
            "Task Description".to_string(),
            1,
            Some(1_640_995_200_000_000_000),
            Some(2.0),
            10,
            Some(9.0),
            Some(11.0),
        );
        
        let tasks = contract.get_tasks(accounts(1));
        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Task Description");
        assert_eq!(task.priority, 1);
        assert_eq!(task.deadline, Some(1_640_995_200_000_000_000));
        assert_eq!(task.estimated_time, Some(2.0));
        assert_eq!(task.reward_points, 10);
        assert_eq!(task.preferred_start_time, Some(9.0));
        assert_eq!(task.preferred_end_time, Some(11.0));
    }

    #[test]
    fn test_update_task() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        contract.add_task(
            "Test Task".to_string(),
            "Task Description".to_string(),
            1,
            Some(1_640_995_200_000_000_000),
            Some(2.0),
            10,
            Some(9.0),
            Some(11.0),
        );

        let tasks = contract.get_tasks(accounts(1));
        let task_id = tasks[0].id;

        contract.update_task(
            task_id,
            "Updated Task".to_string(),
            "Updated Description".to_string(),
            2,
            Some(1_641_995_200_000_000_000),
            Some(3.0),
            20,
            Some(10.0),
            Some(12.0),
        );

        let tasks = contract.get_tasks(accounts(1));
        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.title, "Updated Task");
        assert_eq!(task.description, "Updated Description");
        assert_eq!(task.priority, 2);
        assert_eq!(task.deadline, Some(1_641_995_200_000_000_000));
        assert_eq!(task.estimated_time, Some(3.0));
        assert_eq!(task.reward_points, 20);
        assert_eq!(task.preferred_start_time, Some(10.0));
        assert_eq!(task.preferred_end_time, Some(12.0));
    }

    #[test]
    fn test_remove_task() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        contract.add_task(
            "Test Task".to_string(),
            "Task Description".to_string(),
            1,
            Some(1_640_995_200_000_000_000),
            Some(2.0),
            10,
            Some(9.0),
            Some(11.0),
        );

        let tasks = contract.get_tasks(accounts(1));
        let task_id = tasks[0].id;

        contract.remove_task(task_id);
        let tasks = contract.get_tasks(accounts(1));
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_add_reward() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        contract.add_reward("Reward".to_string(), "Reward Description".to_string(), 50);

        let rewards = contract.get_rewards(accounts(1));
        assert_eq!(rewards.len(), 1);
        let reward = &rewards[0];
        assert_eq!(reward.title, "Reward");
        assert_eq!(reward.description, "Reward Description");
        assert_eq!(reward.cost, 50);
    }

    #[test]
    fn test_remove_reward() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        contract.add_reward("Reward".to_string(), "Reward Description".to_string(), 50);

        let rewards = contract.get_rewards(accounts(1));
        let reward_id = rewards[0].id;

        contract.remove_reward(reward_id);
        let rewards = contract.get_rewards(accounts(1));
        assert_eq!(rewards.len(), 0);
    }

    #[test]
    fn test_redeem_reward() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        contract.add_reward("Reward".to_string(), "Reward Description".to_string(), 50);

        let rewards = contract.get_rewards(accounts(1));
        let reward_id = rewards[0].id;

        contract.account_reward_points.insert(accounts(1), 100);

        let result = contract.redeem_reward(reward_id);
        assert!(result);
        let points = contract.get_account_reward_points(accounts(1));
        assert_eq!(points, 50);
    }

    #[test]
    fn test_update_completed_tasks_per_day() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        contract.add_task(
            "Test Task".to_string(),
            "Task Description".to_string(),
            1,
            Some(1_640_995_200_000_000_000),
            Some(2.0),
            10,
            Some(9.0),
            Some(11.0),
        );

        let tasks = contract.get_tasks(accounts(1));
        let task_id = tasks[0].id;

        contract.mark_complete(task_id);
        let completed_tasks = contract.get_completed_tasks_per_day(accounts(1));
        assert_eq!(completed_tasks.len(), 1);
    }

    #[test]
    fn test_update_working_hours() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        let working_hours = WeeklyWorkingHours {
            monday: WorkingHours { start_time: 9.0, end_time: 17.0 },
            tuesday: WorkingHours { start_time: 9.0, end_time: 17.0 },
            wednesday: WorkingHours { start_time: 9.0, end_time: 17.0 },
            thursday: WorkingHours { start_time: 9.0, end_time: 17.0 },
            friday: WorkingHours { start_time: 9.0, end_time: 17.0 },
            saturday: WorkingHours { start_time: 9.0, end_time: 12.0 },
            sunday: WorkingHours { start_time: 0.0, end_time: 0.0 },
        };

        contract.update_working_hours(working_hours.clone());

        let stored_hours = contract.get_working_hours(accounts(1)).unwrap();
        assert_eq!(stored_hours, working_hours);
    }

    #[test]
    fn test_update_time_slots() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = TodoListV1::new();

        let time_slots = TimeSlots {
            monday: vec![TimeSlot { start_time: 9.0, end_time: 10.0, task_id: Some(1) }],
            tuesday: vec![],
            wednesday: vec![],
            thursday: vec![],
            friday: vec![],
            saturday: vec![],
            sunday: vec![],
        };

        contract.update_time_slots(time_slots.clone());

        let stored_slots = contract.get_time_slots(accounts(1)).unwrap();
        assert_eq!(stored_slots, time_slots);
    }
}

