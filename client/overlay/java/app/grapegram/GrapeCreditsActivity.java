package app.grapegram;

import android.content.Context;
import android.view.View;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import org.telegram.messenger.browser.Browser;
import org.telegram.ui.Cells.TextSettingsCell;

import tw.nekomimi.nekogram.settings.BaseNekoSettingsActivity;

/** Full credits page: upstream projects, verified links, design and licenses. */
public class GrapeCreditsActivity extends BaseNekoSettingsActivity {

    private static final String CHERRYGRAM = "https://github.com/arsLan4k1390/Cherrygram";
    private static final String EXTERAGRAM = "https://github.com/exteraSquad/exteraGram";
    private static final String EXTERALESS = "https://github.com/exteraless/exteraless";

    private int cherrygramRow;
    private int cherrygramChannelRow;
    private int cherrygramSupportRow;
    private int exteraGramRow;
    private int exteraGramChannelRow;
    private int exteraGramChatRow;
    private int exteralessRow;
    private int exteralessChannelRow;
    private int designRow;
    private int studioRow;
    private int cherrySourceRow;
    private int exteraSourceRow;
    private int exteralessSourceRow;
    private int licenseRow;

    @Override
    protected void updateRows() {
        super.updateRows();
        cherrygramRow = addRow();
        cherrygramChannelRow = addRow();
        cherrygramSupportRow = addRow();
        exteraGramRow = addRow();
        exteraGramChannelRow = addRow();
        exteraGramChatRow = addRow();
        exteralessRow = addRow();
        exteralessChannelRow = addRow();
        designRow = addRow();
        studioRow = addRow();
        cherrySourceRow = addRow();
        exteraSourceRow = addRow();
        exteralessSourceRow = addRow();
        licenseRow = addRow();
    }

    @Override
    protected String getActionBarTitle() {
        return "Авторы и благодарности";
    }

    @Override
    protected void onItemClick(View view, int position, float x, float y) {
        String url = null;
        if (position == cherrygramRow || position == cherrygramChannelRow) {
            url = "https://t.me/cherrygram";
        } else if (position == cherrygramSupportRow) {
            url = "https://t.me/CherrygramSupport";
        } else if (position == exteraGramRow || position == exteraGramChannelRow) {
            url = "https://t.me/exteraGram";
        } else if (position == exteraGramChatRow) {
            url = "https://t.me/exteraChat";
        } else if (position == exteralessRow || position == exteralessChannelRow) {
            url = "https://t.me/exteraless";
        } else if (position == designRow) {
            url = "https://t.me/the8055u";
        } else if (position == studioRow) {
            url = "https://t.me/BlueprintDsgn";
        } else if (position == cherrySourceRow) {
            url = CHERRYGRAM;
        } else if (position == exteraSourceRow) {
            url = EXTERAGRAM;
        } else if (position == exteralessSourceRow) {
            url = EXTERALESS;
        }
        if (url != null) {
            Browser.openUrl(getParentActivity(), url);
        }
    }

    @Override
    protected BaseListAdapter createAdapter(Context context) {
        return new ListAdapter(context);
    }

    private class ListAdapter extends BaseListAdapter {

        ListAdapter(Context context) {
            super(context);
        }

        @Override
        public void onBindViewHolder(@NonNull RecyclerView.ViewHolder holder, int position, boolean partial) {
            if (holder.getItemViewType() != TYPE_SETTINGS) {
                return;
            }
            TextSettingsCell cell = (TextSettingsCell) holder.itemView;
            if (position == cherrygramRow) {
                cell.setTextAndValue("Cherrygram", "@Cherrygram", true);
            } else if (position == cherrygramChannelRow) {
                cell.setTextAndValue("Канал Cherrygram", "@cherrygram", true);
            } else if (position == cherrygramSupportRow) {
                cell.setTextAndValue("Поддержка Cherrygram", "@CherrygramSupport", true);
            } else if (position == exteraGramRow) {
                cell.setTextAndValue("exteraGram", "@exteraGram", true);
            } else if (position == exteraGramChannelRow) {
                cell.setTextAndValue("Канал exteraGram", "@exteraGram", true);
            } else if (position == exteraGramChatRow) {
                cell.setTextAndValue("Чат exteraGram", "@exteraChat", true);
            } else if (position == exteralessRow) {
                cell.setTextAndValue("exteraless", "@exteraless", true);
            } else if (position == exteralessChannelRow) {
                cell.setTextAndValue("Канал exteraless", "@exteraless", true);
            } else if (position == designRow) {
                cell.setTextAndValue("Иконки и дизайн", "@the8055u", true);
            } else if (position == studioRow) {
                cell.setTextAndValue("Студия дизайна", "@BlueprintDsgn", true);
            } else if (position == cherrySourceRow) {
                cell.setTextAndValue("Исходники Cherrygram", "GPL-2.0 · GitHub", true);
            } else if (position == exteraSourceRow) {
                cell.setTextAndValue("Исходники exteraGram", "GPL-3.0 · GitHub", true);
            } else if (position == exteralessSourceRow) {
                cell.setTextAndValue("Исходники exteraless", "GPL-3.0 · GitHub", true);
            } else if (position == licenseRow) {
                cell.setText("grapeGram · GPL-3.0. Upstream-проекты не поддерживают grapeGram.", false);
            }
        }

        @Override
        public int getItemViewType(int position) {
            return TYPE_SETTINGS;
        }
    }
}
