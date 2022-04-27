const { SlashCommandBuilder } = require('@discordjs/builders');

module.exports = {
    data: new SlashCommandBuilder()
        .setName('cabbage')
        .setDescription('Cabbage your friends!')
        .addMentionableOption(option => option.setName('target').setDescription('The target you want to cabbage')),
    async execute(interaction) {
        return interaction.reply('wip');
    },
};