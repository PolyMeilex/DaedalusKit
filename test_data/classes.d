const int ATR_INDEX_MAX = 8;
const int MAX_HITCHANCE = 5;
const int PROT_INDEX_MAX = 8;
const int DAM_INDEX_MAX = 8;

class C_NPC {
	var int id;
	var string name[5];
	var string slot;
	var string effect;
	var int npctype;
	var int flags;
	var int attribute[ATR_INDEX_MAX];
	var int hitchance[MAX_HITCHANCE];
	var int protection[PROT_INDEX_MAX];
	var int damage[DAM_INDEX_MAX];
	var int damagetype;
	var int guild;
	var int level;
	var func mission[5];
	var int fight_tactic;
	var int weapon;
	var int voice;
	var int voicepitch;
	var int bodymass;
	var func daily_routine;
	var func start_aistate;
	var string spawnpoint;
	var int spawndelay;
	var int senses;
	var int senses_range;
	var int aivar[100];
	var string wp;
	var int exp;
	var int exp_next;
	var int lp;
	var int bodystateinterruptableoverride;
	var int nofocus;
};
