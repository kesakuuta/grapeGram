package app.grapegram;

import android.content.Context;
import android.view.View;
import android.widget.FrameLayout;

import org.telegram.messenger.R;
import org.telegram.ui.ActionBar.ActionBar;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.ActionBar.Theme;
import org.telegram.ui.Components.IconBackgroundColors;
import org.telegram.ui.Components.LayoutHelper;
import org.telegram.ui.Components.UItem;
import org.telegram.ui.Components.UniversalAdapter;
import org.telegram.ui.Components.UniversalRecyclerView;
import org.telegram.ui.SettingsActivity.SettingCell;

import java.util.ArrayList;

import app.exteraless.ai.ui.AiSettingsActivity;
import app.exteraless.plugins.ui.PluginsActivity;
import app.exteraless.settings.OpenExteraAppearanceActivity;
import app.exteraless.settings.OpenExteraAyuMomentsActivity;
import app.exteraless.settings.OpenExteraChatsActivity;

/** The single grapeGram settings hub, built from Telegram's own settings cells. */
public class GrapeSettingsActivity extends BaseFragment {

    private static final int ID_APPEARANCE = 1;
    private static final int ID_CHATS = 2;
    private static final int ID_AYU = 3;
    private static final int ID_AI = 4;
    private static final int ID_PLUGINS = 5;
    private static final int ID_BADGES = 6;
    private static final int ID_CREDITS = 7;

    private UniversalRecyclerView listView;

    @Override
    public View createView(Context context) {
        setHasOwnBackground(true);

        actionBar.setBackButtonImage(R.drawable.ic_ab_back);
        actionBar.setAllowOverlayTitle(true);
        actionBar.setTitle("grapeGram");
        actionBar.setActionBarMenuOnItemClick(new ActionBar.ActionBarMenuOnItemClick() {
            @Override
            public void onItemClick(int id) {
                if (id == -1) {
                    finishFragment();
                }
            }
        });

        FrameLayout contentView = new FrameLayout(context);
        contentView.setBackgroundColor(getThemedColor(Theme.key_windowBackgroundGray));

        listView = new UniversalRecyclerView(this, this::fillItems, this::onClick, null);
        listView.setSections();
        listView.adapter.setApplyBackground(false);
        listView.setSectionsDrawBackground(true);
        contentView.addView(listView, LayoutHelper.createFrame(LayoutHelper.MATCH_PARENT, LayoutHelper.MATCH_PARENT));
        actionBar.setAdaptiveBackground(listView);

        fragmentView = contentView;
        return fragmentView;
    }

    private void fillItems(ArrayList<UItem> items, UniversalAdapter adapter) {
        items.add(UItem.asHeader("grapeGram " + GrapeConfig.VERSION));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Внешний вид"));
        items.add(row(ID_APPEARANCE, IconBackgroundColors.PURPLE, R.drawable.settings_account,
                "Оформление", "темы, анимации, Monet"));
        items.add(row(ID_CHATS, IconBackgroundColors.BLUE, R.drawable.settings_chat,
                "Чаты", "сообщения, папки, иконка приложения"));
        items.add(row(ID_AYU, IconBackgroundColors.ORANGE, R.drawable.settings_folders,
                "AyuMoments", "истории и статусы"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("Интеллект и расширения"));
        items.add(row(ID_AI, IconBackgroundColors.CYAN, R.drawable.msg_bot,
                "AI", "провайдеры и модели"));
        items.add(row(ID_PLUGINS, IconBackgroundColors.GREEN, R.drawable.settings_premium,
                "Плагины", "установка, разрешения, исходники, AI-анализ, журнал"));
        items.add(UItem.asShadow(null));

        items.add(UItem.asHeader("О приложении"));
        items.add(row(ID_BADGES, IconBackgroundColors.BLUE_DEEP, R.drawable.filled_profile_settings,
                "Бейджи", "подписанный список из GitHub"));
        items.add(row(ID_CREDITS, IconBackgroundColors.ORANGE_DEEP, R.drawable.settings_ask,
                "Авторы и благодарности", "Cherrygram · exteraGram · exteraless"));
    }

    private static UItem row(int id, IconBackgroundColors colors, int icon, String title, String subtitle) {
        return SettingCell.Factory.of(id, colors.top, colors.bottom, icon, title, subtitle);
    }

    private void onClick(UItem item, View view, int position, float x, float y) {
        switch (item.id) {
            case ID_APPEARANCE:
                presentFragment(new OpenExteraAppearanceActivity());
                break;
            case ID_CHATS:
                presentFragment(new OpenExteraChatsActivity());
                break;
            case ID_AYU:
                presentFragment(new OpenExteraAyuMomentsActivity());
                break;
            case ID_AI:
                presentFragment(new AiSettingsActivity());
                break;
            case ID_PLUGINS:
                presentFragment(new PluginsActivity());
                break;
            case ID_BADGES:
                presentFragment(new GrapeBadgesActivity());
                break;
            case ID_CREDITS:
                presentFragment(new GrapeCreditsActivity());
                break;
            default:
                break;
        }
    }

}
