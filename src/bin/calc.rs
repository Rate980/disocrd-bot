use std::collections::HashMap;
use std::hash::Hash;

use chrono::Datelike;
use chrono_tz::Asia;
use discord_bot::{
    message_data::{ChannelData, Emoji, EmojiData, JsonData, MessageData},
    utils::filename,
};
use dotenvy::dotenv;
use itertools::Itertools;
use serenity::all::{ChannelId, EmojiId, GuildId, UserId};

const FIRST_YEAR: usize = 2023;
const YEARS: usize = 2;

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

fn extract_top10<K: Eq + Hash>(counter: Counter<K>) -> Vec<(K, usize)> {
    counter
        .into_iter()
        .sorted_by(|a, b| a.1.cmp(&b.1).reverse())
        .take(10)
        .collect()
}

fn print_data<V, I>(
    i: Option<usize>,
    id: &I,
    count: &usize,
    dates: &HashMap<I, V>,
    channels: &HashMap<ChannelId, ChannelData>,
    par_channels: Option<&CounterPerChannel<I>>,
) where
    V: Default + Clone + std::fmt::Display,
    I: Eq + Hash + Copy,
{
    let data = dates.get(id).cloned().unwrap_or(Default::default());
    if let Some(i) = i {
        print!("{} ", i + 1);
    }
    print!("{}: {}", data, count);
    if let Some(par_channels) = par_channels {
        let par_channel = par_channels.get(id).unwrap();
        print_par_channels(par_channel, channels, *count);
    }
    println!();
}

fn print_par_channels(
    par_channel: &Counter<ChannelId>,
    channels: &HashMap<ChannelId, ChannelData>,
    sum: usize,
) {
    par_channel
        .iter()
        .sorted_by(|a, b| a.1.cmp(b.1).reverse())
        .map(|(key, value)| (key, ((*value as f64 / sum as f64) * 100.0) as usize))
        .sorted_by(|a, b| a.1.cmp(&b.1).reverse())
        .for_each(|(channel_id, parent)| {
            let channel = channels.get(channel_id).cloned().unwrap_or_default();
            print!(" {}: {}%", channel.name, parent);
        });
}

fn print_single_data<V, I>(
    id: &I,
    counter: &HashMap<I, usize>,
    dates: &HashMap<I, V>,
    channels: &HashMap<ChannelId, ChannelData>,
    par_channels: Option<&CounterPerChannel<I>>,
) where
    V: Default + Clone + std::fmt::Display,
    I: Eq + Hash + Copy,
{
    let count = counter.get(id).unwrap();
    print_data(None, id, count, dates, channels, par_channels);
}

fn print_dates<V, I>(
    counter: &[(I, usize)],
    dates: &HashMap<I, V>,
    channels: &HashMap<ChannelId, ChannelData>,
    par_channels: Option<&CounterPerChannel<I>>,
) where
    V: Default + Clone + std::fmt::Display,
    I: Eq + Hash + Copy,
{
    counter
        .iter()
        .enumerate()
        .for_each(|(i, (id, count))| print_data(Some(i), id, count, dates, channels, par_channels));
}

fn print_emojis(
    counter: &[(Emoji, usize)],
    emojis: &HashMap<EmojiId, EmojiData>,
    channels: &HashMap<ChannelId, ChannelData>,
    par_channels: Option<&EmojiCounterPerChannel>,
) {
    counter.iter().enumerate().for_each(|(i, (emoji, count))| {
        let output = match emoji {
            Emoji::Custom(id) => match emojis.get(id).cloned() {
                Some(emoji_data) => emoji_data.to_string(),
                None => "Unknown".to_string(),
            },
            Emoji::Unicode(name) => name.clone(),
            _ => "Unknown".to_string(),
        };
        print!("{} {}: {}", i + 1, output, count);
        if let Some(par_channels) = par_channels {
            let par_channel = par_channels.get(emoji).unwrap();
            print_par_channels(par_channel, channels, *count);
        }

        println!()
    });
}

fn main() {
    dotenv().unwrap();
    let guild_id = std::env::var("GUILD_ID")
        .unwrap()
        .parse::<GuildId>()
        .unwrap();
    let file = std::fs::File::open(filename(guild_id)).unwrap();
    let data: JsonData = serde_json::from_reader(file).unwrap();

    let members = data.members;
    let channels = data.channels;

    let not_include_channels: Vec<ChannelId> =
        vec![869111104243122187.into(), 1095933862657405041.into()];

    let mut user_message_sums: Counters<UserCounter> = Default::default();
    let mut user_message_sum_par_channels: Counters<UserCounterPerChannel> = Default::default();

    let mut user_mention_sums: Counters<UserCounter> = Default::default();
    let mut user_mention_sum_par_channels: Counters<UserCounterPerChannel> = Default::default();

    let mut emoji_sums: Counters<EmojiCounter> = Default::default();
    let mut emoji_sum_per_channels: Counters<EmojiCounterPerChannel> = Default::default();
    let mut reaction_sums: Counters<UserCounter> = Default::default();

    let mut message_sum: Counters<usize> = Default::default();
    let mut mention_sum: Counters<usize> = Default::default();

    data.messages.iter().for_each(|(channel_id, messages)| {
        if not_include_channels.contains(channel_id) {
            return;
        }
        messages.iter().for_each(|message| {
            if let Some(user) = members.get(&message.author_id) {
                if user.is_bot {
                    return;
                }
            } else {
                return;
            }
            let time = message.send_time.with_timezone(&Asia::Tokyo);
            if (time.year() as usize) < FIRST_YEAR {
                return;
            }
            let index = time.year() as usize - FIRST_YEAR;
            if index >= YEARS {
                return;
            }

            message_sum[index] += 1;
            mention_sum[index] += message.mentions.len();

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
        let emoji_sum_per_channels = &emoji_sum_per_channels[i];
        let reaction_sum = extract_top10(reaction_sums[i].clone());
        let emoji_custom_usm = emoji_sums[i]
            .iter()
            // .filter(|(emoji, _)| matches!(emoji, Emoji::Custom(_)))
            .map(|(emoji, count)| (emoji.clone(), *count))
            .sorted_by(|a, b| a.1.cmp(&b.1).reverse())
            .enumerate()
            .filter(|(_, (emoji, _))| matches!(emoji, Emoji::Custom(_)))
            .collect::<Vec<_>>();

        println!();
        println!("Year: {}", FIRST_YEAR + i);
        println!("Messages: {}", message_sum);
        println!("Mentions: {}", mention_sum[i]);
        println!();
        println!("message count");
        print_dates(
            &user_message_sum,
            &members,
            &channels,
            Some(user_message_sum_per_channels),
        );
        println!();
        println!("mention count");
        print_dates(
            &user_mention_sum,
            &members,
            &channels,
            Some(user_mention_sum_per_channels),
        );
        println!();
        println!("emoji count");
        print_emojis(
            &emoji_sum,
            &data.emojis,
            &channels,
            Some(emoji_sum_per_channels),
        );
        println!();
        println!("reaction count");
        print_dates(&reaction_sum, &members, &channels, None);
        println!();
        println!("emoji custom count");
        emoji_custom_usm.iter().for_each(|(i, (emoji, count))| {
            let output = match emoji {
                Emoji::Custom(id) => match data.emojis.get(id).cloned() {
                    Some(emoji_data) => emoji_data.to_string(),
                    None => "Unknown".to_string(),
                },
                Emoji::Unicode(name) => name.clone(),
                _ => "Unknown".to_string(),
            };
            println!("{} {}: {}", i + 1, output, count);
        });
        println!();
        let id = 860382628304650240.into();

        print!("Message: ");
        print_single_data(
            &id,
            &user_message_sums[i],
            &members,
            &channels,
            Some(user_message_sum_per_channels),
        );

        print!("Mention: ");
        print_single_data(
            &id,
            &user_mention_sums[i],
            &members,
            &channels,
            Some(user_mention_sum_per_channels),
        );
    }
}
