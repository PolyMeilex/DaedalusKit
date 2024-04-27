instance PC_HERO(C_Npc) {
    attribute[0] = 20;
    attribute[1] = 40;
    Mdl_SetVisual(PC_HERO, "HUMANS.MDS");
	Mdl_SetVisualBody(self, "hum_body_Naked0", 9, 0, "Hum_Head_Pony", 18, 0, -1);
};

func void startup_global() {};
func void init_global() {};
