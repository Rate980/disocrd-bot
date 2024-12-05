use std::collections::HashMap;
use std::hash::Hash;

use chrono::Datelike;
use chrono_tz::Asia;
use discord_bot::message_data::{ChannelData, Emoji, JsonData, MessageData, UserData};
use itertools::Itertools;
use serenity::all::{ChannelId, UserId};

const FIRST_YEAR: usize = 2021;
const YEARS: usize = 4;
const DEFAULT_NAME: &str = "Unknown";
static NOT_INCLUDE_CHANNELS: [u64; 1] = [869111104243122187];

type Counter<K> = HashMap<K, usize>;

type ChannelCounter = Counter<ChannelId>;

type CounterPerChannel<K> = HashMap<K, ChannelCounter>;

type UserCounter = Counter<UserId>;
type UserCounterPerChannel = CounterPerChannel<UserId>;

type EmojiCounter = Counter<Emoji>;
type EmojiCounterPerChannel = CounterPerChannel<Emoji>;

type Counters<T> = [T; YEARS];

fn calc_messages(
    message: &MessageData,
    user_message_sum: &mut UserCounter,
    user_message_sum_per_channels: &mut UserCounterPerChannel,
) {
    let user_id = message.author_id;
    let message_count = user_message_sum.entry(user_id).or_default();
    let message_count_per_channel = user_message_sum_per_channels
        .entry(user_id)
        .or_default()
        .entry(message.channel_id)
        .or_default();

    *message_count_per_channel += 1;
    *message_count += 1;
}

fn calc_mention(
    message: &MessageData,
    user_mention_sum: &mut UserCounter,
    user_mention_sum_per_channels: &mut UserCounterPerChannel,
) {
    message.mentions.iter().for_each(|mention| {
        let mention_count = user_mention_sum.entry(*mention).or_default();
        let mention_count_per_channel = user_mention_sum_per_channels
            .entry(*mention)
            .or_default()
            .entry(message.channel_id)
            .or_default();

        *mention_count += 1;
        *mention_count_per_channel += 1;
    });
}

fn calc_emojis(
    message: &MessageData,
    emoji_sum: &mut EmojiCounter,
    emoji_sum_per_channels: &mut EmojiCounterPerChannel,
    reaction_sum: &mut UserCounter,
) {
    let reaction_counter = reaction_sum.entry(message.author_id).or_default();
    message.reactions.iter().for_each(|(emoji, count)| {
        let count = *count as usize;
        let emoji_count = emoji_sum.entry(emoji.clone()).or_default();
        let emoji_count_per_channel = emoji_sum_per_channels
            .entry(emoji.clone())
            .or_default()
            .entry(message.channel_id)
            .or_default();

        *emoji_count += count;
        *emoji_count_per_channel += count;
        *reaction_counter += count;
    });
}

fn calc_parentage<K: Eq + Hash + Copy>(sum: usize, counter: &Counter<K>) -> HashMap<K, f64> {
    counter
        .iter()
        .map(|(key, value)| (*key, *value as f64 / sum as f64))
        .collect()
}

fn extract_top10<K: Eq + Hash>(counter: Counter<K>) -> Vec<(K, usize)> {
    counter
        .into_iter()
        .sorted_by(|a, b| a.1.cmp(&b.1).reverse())
        // .take(10)
        .collect()
}

fn print_users(
    counter: &[(UserId, usize)],
    members: &HashMap<UserId, UserData>,
    channels: &HashMap<ChannelId, ChannelData>,
    par_channels: Option<&UserCounterPerChannel>,
) {
    counter
        .iter()
        .enumerate()
        .for_each(|(i, (user_id, count))| {
            let member = members.get(user_id).cloned().unwrap_or(UserData::new(
                *user_id,
                DEFAULT_NAME.to_string(),
                DEFAULT_NAME.to_string(),
                None,
            ));
            print!("{} {}: {}", i + 1, member.display_name, count);
            if let Some(par_channels) = par_channels {
                let par_channel = par_channels.get(user_id).unwrap();
                par_channel
                    .iter()
                    .sorted_by(|a, b| a.1.cmp(b.1).reverse())
                    .map(|(key, value)| (key, ((*value as f64 / *count as f64) * 100.0) as usize))
                    .for_each(|(channel_id, parent)| {
                        let channel =
                            channels
                                .get(channel_id)
                                .cloned()
                                .unwrap_or(ChannelData::new(
                                    *channel_id,
                                    DEFAULT_NAME.to_string(),
                                    Default::default(),
                                    Default::default(),
                                ));
                        print!(" {}: {}%", channel.name, parent);
                    });
            }
            println!();
        });
}

fn main() {
    let file = std::fs::File::open("messages.json").unwrap();
    let data: JsonData = serde_json::from_reader(file).unwrap();

    let members = data.members;
    let channels = data.channels;

    let mut user_message_sums: Counters<UserCounter> = Default::default();
    let mut user_message_sum_par_channels: Counters<UserCounterPerChannel> = Default::default();

    let mut user_mention_sums: Counters<UserCounter> = Default::default();
    let mut user_mention_sum_par_channels: Counters<UserCounterPerChannel> = Default::default();

    let mut emoji_sums: Counters<EmojiCounter> = Default::default();
    let mut emoji_sum_per_channels: Counters<EmojiCounterPerChannel> = Default::default();
    let mut reaction_sums: Counters<UserCounter> = Default::default();

    let mut message_sum: Counters<usize> = Default::default();

    data.messages.iter().for_each(|(_channel_id, messages)| {
        if NOT_INCLUDE_CHANNELS.contains(&_channel_id.get()) {
            return;
        }
        messages.iter().for_each(|message| {
            let time = message.send_time.with_timezone(&Asia::Tokyo);
            if (time.year() as usize) < FIRST_YEAR {
                return;
            }
            let index = time.year() as usize - FIRST_YEAR;
            if index >= YEARS {
                return;
            }

            message_sum[index] += 1;

            calc_messages(
                message,
                &mut user_message_sums[index],
                &mut user_message_sum_par_channels[index],
            );

            calc_mention(
                message,
                &mut user_mention_sums[index],
                &mut user_mention_sum_par_channels[index],
            );

            calc_emojis(
                message,
                &mut emoji_sums[index],
                &mut emoji_sum_per_channels[index],
                &mut reaction_sums[index],
            );
        });
    });

    for i in 0..YEARS {
        let message_sum = message_sum[i];
        let user_message_sum = extract_top10(user_message_sums[i].clone());
        let user_message_sum_per_channels = &user_message_sum_par_channels[i];
        let user_mention_sum = extract_top10(user_mention_sums[i].clone());
        let user_mention_sum_per_channels = &user_mention_sum_par_channels[i];
        let emoji_sum = extract_top10(emoji_sums[i].clone());

        println!();
        println!("Year: {}", FIRST_YEAR + i);
        println!("Messages: {}", message_sum);
        println!();
        println!("message count");
        print_users(
            &user_message_sum,
            &members,
            &channels,
            Some(user_message_sum_per_channels),
        );
        println!();
        println!("mention count");
        print_users(
            &user_mention_sum,
            &members,
            &channels,
            Some(user_mention_sum_per_channels),
        );
        println!();
    }
}
